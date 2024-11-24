use actix_web::{web, HttpResponse, Error as AWError};

use crate::database::sqlite::Pool;
use crate::model::scan::ScanRequest;
use crate::model::common::RestResponse;


pub async fn scan_files(requst_json: web::Json<ScanRequest>, db: web::Data<Pool>) -> Result<HttpResponse, AWError>  {
    
    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(Some(output_pwd))))
}