use crate::{package_info::models::PackageInfo, AppState, MyError};
use std::sync::Arc;

use actix_web::{web, HttpResponse};

// 就是个单纯的把数据库的数据以json形式传送前端中
pub async fn get_package_info(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, MyError> {
    let db_pool = &state.db_pool;
    let package_info: Vec<PackageInfo> = sqlx::query_as(
        "SELECT source_ip, source_port, destination_port, proto_type FROM package_info",
    )
    .fetch_all(db_pool)
    .await?;

    Ok(HttpResponse::Ok().json(package_info))
}
