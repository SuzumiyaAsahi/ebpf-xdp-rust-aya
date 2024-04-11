use crate::{
    block_ip::models::{BlockedIp, WriteIp},
    errors::MyError,
    kill_restart::kill_and_restart::kill_and_restart,
    AppState,
};
use actix_web::{web, HttpResponse};
use regex::Regex;
use std::sync::Arc;

pub async fn write_block_ip(
    state: web::Data<Arc<AppState>>,
    write_ip: web::Json<WriteIp>,
) -> Result<HttpResponse, MyError> {
    let db_pool = &state.db_pool;

    // 检查是否有IP地址传入
    let write_ip = match &write_ip.ipv4 {
        Some(ipv4) => ipv4,
        None => return Err(MyError::InvalidInput("请提供要被封锁的ipv4地址".into())),
    };

    // 正则检查，检查传入的IP地址是否符合ipv4规范
    let re = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();

    // 如果不匹配就返回错误
    if !re.is_match(write_ip) {
        return Err(MyError::InvalidInput(
            "输入的IP地址并不是符合ipv4规范的IP地址".to_string(),
        ));
    }

    // 检查一下这个IP地址是否已经存在
    let ip_stored: Vec<BlockedIp> = sqlx::query_as("SELECT id, ipv4 FROM blocked_ip")
        .fetch_all(db_pool)
        .await?;

    let mut ip: Vec<String> = Vec::new();
    for i in ip_stored {
        ip.push(i.ipv4);
    }

    if ip.contains(write_ip) {
        return Err(MyError::DBError("该ip地址已经存在于数据库中了".to_string()));
    }

    // 将IP地址存入到数据库中
    let _ = sqlx::query("INSERT INTO blocked_ip (ipv4) VALUES ($1)")
        .bind(write_ip)
        .execute(db_pool)
        .await?;

    kill_and_restart(state.clone()).await?;

    Ok(HttpResponse::Ok().json("ipv4地址添加成功, ebpf程序也已经关闭并重启过".to_string()))
}
