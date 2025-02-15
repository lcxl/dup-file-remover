use actix_session::Session;
use actix_web::{post, web, Error as AWError, HttpResponse};
use log::{error, info};

use crate::{
    model::login::{LoginParams, LoginResult},
    Settings,
};

#[utoipa::path(
    summary = "Login user account",
    request_body(content = LoginParams),
    responses(
        (status = 200, description = "Login result", body=LoginResult),
        (status = 403, description = "Illegal username or password"),
    ),
)]
#[post("/api/login/account")]
pub async fn login_account(
    requst_json: web::Json<LoginParams>,
    settings: web::Data<Settings>,
    session: Session,
) -> Result<HttpResponse, AWError> {
    let params = requst_json.into_inner();
    if settings.login_user_name != params.username {
        error!("Username does not match");
        return Ok(HttpResponse::Forbidden().body("Illegal username or password"));
    }
    if settings.login_password != params.password {
        error!("Password does not match");
        return Ok(HttpResponse::Forbidden().body("Illegal username or password"));
    }
    let result = LoginResult {
        status: String::from("success"),
        login_type: params.login_type,
        current_authority: String::from("success"),
    };
    session.insert("user", &params.username).unwrap(); // Store user information in session
    info!("Login successful, username: {}", params.username);
    Ok(HttpResponse::Ok().json(result))
}
