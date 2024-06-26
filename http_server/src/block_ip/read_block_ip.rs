use crate::{block_ip::models::BlockedIp, errors::MyError, AppState};
use actix_web::{web, HttpResponse};
use std::sync::Arc;

// 这个没啥可说的，就是单纯的读数据，发数据而已
pub async fn read_block_ip(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, MyError> {
    let db_pool = &state.db_pool;
    let ip_addr: Vec<BlockedIp> = sqlx::query_as("SELECT id, ipv4 FROM blocked_ip")
        .fetch_all(db_pool)
        .await?;

    Ok(HttpResponse::Ok().json(ip_addr))
}
