use crate::{
    errors::ErrorResponse,
    handlers::*,
    models::{
        Comment, CreateComment, CreatePost, LoginUser, PaginatedResponse, Post, RegisterUser,
        TokenResponse,
    },
    routes::*,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        root,
        register,
        login,
        get_posts,
        create_post,
        get_post_by_id,
        update_post,
        delete_post,
        get_comments_for_post,
        create_comment_for_post,
        update_comment,
        delete_comment,
    ),
    components(
        schemas(
            Post,
            CreatePost,
            Comment,
            CreateComment,
            PaginatedResponse<Post>,
            ErrorResponse,
            RegisterUser,
            LoginUser,
            TokenResponse
        )
    ),
    tags(
        (name = "Rust Blog API", description = "一个用 Rust 和 Axum 构建的简单博客 API"),
        (name = "Posts", description = "关于文章的操作"),
        (name = "Comments", description = "关于评论的操作")
    )
)]
pub struct ApiDoc;
