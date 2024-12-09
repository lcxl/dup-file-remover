use serde::Serialize;

#[derive(Serialize)]
pub struct RestResponse<T> {
    success: bool,
    code: i32,
    message: Option<String>,
    data: Option<T>,
}

impl<T> RestResponse<T> {
    pub fn succeed() -> Self {
        RestResponse {
            success: true,
            code: 0,
            message: None,
            data: None,
        }
    }

    pub fn succeed_with_data(data: Option<T>) -> Self {
        RestResponse {
            success: true,
            code: 0,
            message: None,
            data,
        }
    }

    pub fn failed(code: i32, message: Option<String>) -> Self {
        RestResponse {
            success: false,
            code,
            message,
            data: None,
        }
    }

    pub fn failed_with_data(code: i32, message: Option<String>, data: Option<T>) -> Self {
        RestResponse {
            success: false,
            code,
            message,
            data,
        }
    }
}

/*
impl RestResponse<String> {
    pub fn succeed() -> Self {
        RestResponse {
            succeed: true,
            code: 0,
            message: None,
            data: None
        }
    }
    pub fn failed(code: i32, message: Option<String>) -> Self {
        RestResponse {
            success: false,
            code,
            message,
            data: None,
        }
    }
}
     */
