use crate::{AppState, MyError};
use std::sync::Arc;

use actix_web::{web, HttpResponse};

pub async fn http_not_found(_state: web::Data<Arc<AppState>>) -> Result<HttpResponse, MyError> {
    Ok(HttpResponse::NotFound().json("笨蛋，访问地址写错了".to_string()))
}
