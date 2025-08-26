use crate::errors::AppError;
use validator::{Validate, ValidationErrors};

pub trait ValidatedJson<T> {
    fn validate_json(self) -> Result<T, AppError>;
}

impl<T> ValidatedJson<T> for axum::Json<T>
where
    T: Validate,
{
    fn validate_json(self) -> Result<T, AppError> {
        match self.0.validate() {
            Ok(()) => Ok(self.0),
            Err(validation_errors) => Err(AppError::validation(format_validation_errors(
                &validation_errors,
            ))),
        }
    }
}

pub fn format_validation_errors(validation_errors: &ValidationErrors) -> String {
    let errors: Vec<String> = validation_errors
        .field_errors()
        .into_iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(move |error| {
                format!(
                    "{}: {}",
                    field,
                    error.message.as_ref().unwrap_or(&"Invalid value".into())
                )
            })
        })
        .collect();

    format!("验证失败: {}", errors.join(", "))
}
