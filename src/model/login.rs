use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Login params
#[derive(Deserialize, ToSchema)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
    #[serde(rename(deserialize = "autoLogin"))]
    #[schema(rename="autoLogin")]
    pub auto_login: bool,
    #[serde(rename(deserialize = "type"))]
    #[schema(rename="type")]
    pub login_type: String,
}

#[derive(Deserialize, ToSchema)]
pub struct FakeCaptchaParams {
   pub phone: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct FakeCaptcha {
  pub code: Option<u32>,
  pub status: Option<String>,
}


#[derive(Serialize, ToSchema)]
pub struct LoginResult {
    pub status: String,
    #[serde(rename(serialize = "type"))]
    #[schema(rename="type")]
    pub login_type: String,
    #[serde(rename(serialize = "currentAuthority"))]
    #[schema(rename="currentAuthority")]
    pub current_authority: String,
}
