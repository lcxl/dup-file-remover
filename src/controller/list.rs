use actix_web::{get, web, Error as AWError, HttpResponse};

use crate::{
    database::{file_info::FileInfoList, sqlite::PoolDatabaseManager},
    model::{
        common::{ErrorCode, RestResponse},
        list::QueryListParams,
    },
};
#[utoipa::path(
    summary = "List files",
    params(QueryListParams),
    responses(
        (status = 200, description = "The list of file info with md5 count", body=FileInfoList)
    ),
)]
#[get("/list")]
pub async fn list_files(
    query_list: web::Query<QueryListParams>,
    db: web::Data<PoolDatabaseManager>,
) -> Result<HttpResponse, AWError> {
    let conn = db.get_ref();
    let result = conn.0.list_files(&query_list);
    if result.is_err() {
        return Ok(HttpResponse::Ok().json(RestResponse::failed(
            ErrorCode::UNKNOWN_ERROR,
            result.err().unwrap().to_string(),
        )));
    }
    Ok(HttpResponse::Ok().json(result.unwrap()))
}
