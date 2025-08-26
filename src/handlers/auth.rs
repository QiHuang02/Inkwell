use crate::models::{Claims, LoginUser, RegisterUser, TokenResponse, User};
use crate::{
    errors::{AppError, ErrorResponse},
    models::AppState,
    utils::{hash_password, verify_password},
    validation::ValidatedJson,
};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer "));

    let token = match token {
        Some(token) => token,
        None => {
            return AppError::authentication("Missing Authorization header").into_response();
        }
    };

    let decoding_key = DecodingKey::from_secret(state.config.jwt_secret.as_ref());

    let token_data = match decode::<Claims>(token, &decoding_key, &Validation::default()) {
        Ok(data) => data,
        Err(_) => {
            return AppError::authentication("Invalid or expired token").into_response();
        }
    };

    // 将解码的用户信息添加到请求扩展中，供后续处理器使用
    req.extensions_mut().insert(token_data.claims);

    next.run(req).await
}

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterUser,
    responses(
        (status = 201, description = "用户注册成功"),
        (status = 409, description = "用户名已存在", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<AppState>,
    json_payload: Json<RegisterUser>,
) -> Result<StatusCode, AppError> {
    let payload = json_payload.validate_json()?;
    // 使用 bcrypt 哈希密码，使用 spawn_blocking 在单独线程中执行以避免阻塞异步运行时
    let password_hash = hash_password(&payload.password).await?;

    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&payload.username)
        .bind(&password_hash)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return Err(AppError::conflict("用户名已存在"));
            }
            Err(e.into())
        }
    }
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginUser,
    responses(
        (status = 200, description = "用户登录成功", body = TokenResponse),
        (status = 401, description = "用户名或密码错误", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    json_payload: Json<LoginUser>,
) -> Result<Json<TokenResponse>, AppError> {
    let payload = json_payload.validate_json()?;
    // 1. 根据用户名从数据库查找用户
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.pool) // fetch_optional 返回 Option<User>
        .await?
        .ok_or_else(|| AppError::authentication("用户名或密码错误"))?;

    // 2. 验证密码 - 使用 spawn_blocking 避免阻塞异步运行时
    let password_valid = verify_password(&payload.password, &user.password_hash).await?;

    if !password_valid {
        return Err(AppError::authentication("用户名或密码错误"));
    }

    // 3. 生成 JWT
    let claims = Claims {
        sub: user.username,
        role: user.role,
        exp: (Utc::now() + Duration::days(state.config.jwt_expiration_days)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
    )?;

    Ok(Json(TokenResponse { token }))
}
