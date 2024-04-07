use crate::{
    block_ip::models::{BlockedIp, WriteIp},
    errors::MyError,
    AppState,
};
use actix_web::{web, HttpResponse};
use regex::Regex;
use std::sync::Arc;

pub async fn delete_block_ip(
    state: web::Data<Arc<AppState>>,
    delete_ip: web::Json<WriteIp>,
) -> Result<HttpResponse, MyError> {
    let db_pool = &state.db_pool;
    let delete_ip = match &delete_ip.ipv4 {
        Some(ipv4) => ipv4,
        None => return Err(MyError::InvalidInput("请提供要封锁的ipv4地址".into())),
    };

    let re = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();

    if !re.is_match(delete_ip) {
        return Err(MyError::InvalidInput(
            "输入的IP地址并不是符合ipv4规范的IP地址".to_string(),
        ));
    }

    let ip_stored: Vec<BlockedIp> = sqlx::query_as("SELECT id, ipv4 FROM blocked_ip")
        .fetch_all(db_pool)
        .await?;

    let mut ip: Vec<String> = Vec::new();
    for i in ip_stored {
        ip.push(i.ipv4);
    }

    if !ip.contains(delete_ip) {
        return Err(MyError::DBError("该ip地址并不存在于数据库中".to_string()));
    }

    let _ = sqlx::query("DELETE FROM blocked_ip WHERE ipv4 = $1")
        .bind(delete_ip)
        .execute(db_pool)
        .await?;

    Ok(HttpResponse::Ok().json("ipv4地址删除成功".to_string()))
}
