use actix_session::Session;
use actix_web::{get, Error as AWError, HttpResponse};
use log::{info, warn};

use crate::model::user::{CurrentUser, NoticeIconList};

pub const SESSION_KEY_USERNAME: &str = "username";
pub const USER_ADMIN: &str = "admin";
#[utoipa::path(
    summary = "Get current user",
    responses(
        (status = 200, description = "Current user info", body = CurrentUser),
        (status  = 401, description = "Unauthorized"),
    ),
)]
#[get("/api/currentUser")]
pub async fn get_current_user(session: Session) -> Result<HttpResponse, AWError> {
    if let Some(username) = session.get::<String>(SESSION_KEY_USERNAME)? {
        let current_user = CurrentUser {
            name: Some(username),
            avatar: None,
            userid: None,
            email: None,
            signature: None,
            title: None,
            group: None,
            tags: None,
            notify_count: None,
            unread_count: None,
            country: None,
            access: Some(USER_ADMIN.to_string()),
            geographic: None,
            address: None,
            phone: None,
        };
        info!("Current user: {:?}", current_user);
        return Ok(HttpResponse::Ok().json(current_user));
    }
    warn!("User is not logged in.");
    return Ok(HttpResponse::Unauthorized().body("User is not logged in."));
}

#[utoipa::path(
    summary = "Get notices",
    responses(
        (status = 200, description = "Notices", body = NoticeIconList),
        (status  = 401, description = "Unauthorized"),
    ),
)]
#[get("/api/notices")]
pub async fn get_notices(session: Session) -> Result<HttpResponse, AWError> {
    let user = session.get::<String>(SESSION_KEY_USERNAME)?;
    if user.is_none() {
        return Ok(HttpResponse::Unauthorized().body("User is not logged in."));
    }
    let username = user.unwrap();
    info!("Fetching notices for user: {}", username);

    // Simulate fetching notices for the user
    let notice_icon_list = NoticeIconList {
        data: None,
        total: 0,
        success: true,
    };
    return Ok(HttpResponse::Ok().json(notice_icon_list));
}
