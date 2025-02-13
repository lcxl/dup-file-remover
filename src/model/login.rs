use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Login params
#[derive(Deserialize, ToSchema)]
pub struct LoginParams  {
    pub username: String,
    pub password: String,
    #[serde(rename(deserialize = "autoLogin"))]
    pub auto_login: bool,
    #[serde(rename(deserialize = "type"))]
    pub login_type: String,
  }

  #[derive(Serialize, ToSchema)]
  pub struct  LoginResult  {
    pub status: String,
    #[serde(rename(serialize = "type"))]
    pub login_type: String,
    #[serde(rename(serialize = "currentAuthority"))]
    pub current_authority: String,
  }