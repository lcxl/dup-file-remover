use actix_web::{post, web, Error as AWError, HttpResponse};

use crate::
    model::login::{LoginParams, LoginResult}
;

#[utoipa::path(
    summary = "Login user account",
    request_body(content = LoginParams),
    responses(
        (status = 200, description = "Login result", body=LoginResult)
    ),
)]
#[post("/api/login/account")]
pub async fn login_account(
    requst_json: web::Json<LoginParams>,
) -> Result<HttpResponse, AWError> {
    let params = requst_json.into_inner();
    let result = LoginResult {
        status: String::from("success"),
        login_type: params.login_type,
        current_authority: String::from("success"),
    };

    Ok(HttpResponse::Ok().json(result))
}
