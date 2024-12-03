use actix_web::{web, HttpResponse, Error as AWError};

use crate::database::sqlite::{Pool, PoolDatabaseManager};
use crate::model::scan::ScanRequest;
use crate::model::common::RestResponse;


pub async fn scan_files(requst_json: web::Json<ScanRequest>, db: web::Data<PoolDatabaseManager>) -> Result<HttpResponse, AWError>  {
    let test_none_response: RestResponse<()> = RestResponse::succeed();
    HttpResponse::Ok().json(test_none_response);
    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(None::<()>)))
}