use std::result;

use actix_web::{get, web, Error as AWError, HttpResponse};

use crate::{database::{file_info::FileInfoWithMd5Count, sqlite::PoolDatabaseManager}, model::{common::{ErrorCode, RestResponse}, list::QueryListParams}};
#[utoipa::path(
    summary = "List files",
    params(QueryListParams),
    responses(
        (status = 200, description = "The list of file info with md5 count", body=Vec<FileInfoWithMd5Count>)
    ),
)]
#[get("/api/list")]
pub async fn list_files(
    query_list: web::Query<QueryListParams>,
    db: web::Data<PoolDatabaseManager>,
) -> Result<HttpResponse, AWError> {
    let conn = db.get_ref();
    let min_file_size:i64 = query_list.min_file_size.unwrap_or(0);
    let max_file_size:i64  = query_list.max_file_size.unwrap_or(std::i64::MAX);
    let result = conn.0.list_files(query_list.page_no, query_list.page_count, min_file_size, max_file_size);
    if result.is_err() {
        return Ok(HttpResponse::Ok().json(RestResponse::failed(ErrorCode::UNKNOWN_ERROR, result.err().unwrap().to_string())));
    }
    Ok(HttpResponse::Ok().json(result.unwrap()))
}