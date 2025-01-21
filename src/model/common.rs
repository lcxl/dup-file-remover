use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct RestResponse<T : ToSchema> {
    success: bool,
    code: i32,
    message: Option<String>,
    data: Option<T>,
}
pub struct ErrorCode(i32);

impl ErrorCode {
    pub const SUCCESS: ErrorCode = ErrorCode(0);
    pub const UNKNOWN_ERROR: ErrorCode = ErrorCode(1);
    pub const FILE_PATH_NOT_FOUND: ErrorCode = ErrorCode(11);
}

impl RestResponse<()> {
    pub fn succeed() -> Self {
        RestResponse {
            success: true,
            code: ErrorCode::SUCCESS.0,
            message: None,
            data: None,
        }
    }

    pub fn failed(error_code: ErrorCode, message: String) -> Self {
        RestResponse {
            success: false,
            code: error_code.0,
            message: Some(message),
            data: None,
        }
    }
}

impl<T : ToSchema> RestResponse<T> {

    pub fn succeed_with_data(data: Option<T>) -> Self {
        RestResponse {
            success: true,
            code: ErrorCode::SUCCESS.0,
            message: None,
            data,
        }
    }

    pub fn failed_with_data(error_code: ErrorCode, message: Option<String>, data: Option<T>) -> Self {
        RestResponse {
            success: false,
            code: error_code.0,
            message,
            data,
        }
    }
}
