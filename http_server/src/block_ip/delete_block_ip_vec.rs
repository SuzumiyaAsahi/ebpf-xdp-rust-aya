use crate::{
    block_ip::models::{BlockedIp, WriteIp},
    errors::MyError,
    kill_restart::kill_and_restart::kill_and_restart,
    AppState,
};
use actix_web::{web, HttpResponse};
use regex::Regex;
use std::{collections::HashSet, sync::Arc};

pub async fn delete_block_ip_vec(
    state: web::Data<Arc<AppState>>,
    web::Json(delete_ip): web::Json<Vec<WriteIp>>,
) -> Result<HttpResponse, MyError> {
    // 检查一下输入的数据是否有重复项
    let mut ips_set = HashSet::new();

    for ip_entry in &delete_ip {
        if !ips_set.insert(&ip_entry.ipv4) {
            return Err(MyError::InvalidInput("传入的数据中有重复IP地址".into()));
        }
    }

    let db_pool = &state.db_pool;

    // 判断传入的IP数组是否为空
    if delete_ip.is_empty() {
        return Ok(HttpResponse::Ok().json("删除失败，传入的IP数量为零,请填写要删除的IP地址"));
    }

    // 正则检查，检查传入的IP地址是否符合ipv4规范
    let re = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();

    let mut delete_ip_vec: Vec<String> = Vec::new();

    for i in delete_ip {
        if let Some(ip_addr) = i.ipv4 {
            delete_ip_vec.push(ip_addr.clone());
        }
    }

    // 再检查一下，看看是不是传的都是空的数据
    if delete_ip_vec.is_empty() {
        return Ok(HttpResponse::Ok().json("删除失败，传入的IP数量为零,请填写要删除的IP地址"));
    }

    // 现在的封锁IP数据列表
    let ip_stored: Vec<BlockedIp> = sqlx::query_as("SELECT id, ipv4 FROM blocked_ip")
        .fetch_all(db_pool)
        .await?;
    let mut ip: Vec<String> = Vec::new();
    for i in ip_stored {
        ip.push(i.ipv4);
    }

    // 必须所有输入的IP地址都要过检查，要不然就一个都不要过
    // 感觉好.......
    for i in delete_ip_vec.clone() {
        // 如果不匹配就返回错误
        if !re.is_match(i.as_str()) {
            return Err(MyError::InvalidInput(
                format!("删除失败，输入的{}地址并不是符合ipv4规范的IP地址", i).to_string(),
            ));
        }
        // 检查一下这个IP地址是否存在于数据库中
        if !ip.contains(&i) {
            return Err(MyError::DBError(
                format!("删除失败，{}并不存在于数据库中了", i).to_string(),
            ));
        }
    }

    for i in delete_ip_vec {
        let _ = sqlx::query("DELETE FROM blocked_ip WHERE ipv4 = $1")
            .bind(i)
            .execute(db_pool)
            .await?;
    }

    kill_and_restart(state.clone()).await?;

    Ok(HttpResponse::Ok().json("ipv4地址成批地删除成功, 并且ebpf程序已经重新启动".to_string()))
}
