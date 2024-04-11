use crate::{errors::MyError, kill_restart::kill_and_restart::kill_and_restart, AppState};
use actix_web::{web, HttpResponse};
use std::sync::Arc;

pub async fn flush_all(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, MyError> {
    let db_pool = &state.db_pool;

    let _ = sqlx::query("DELETE FROM blocked_ip")
        .execute(db_pool)
        .await?;

    let _ = sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'blocked_ip'")
        .execute(db_pool)
        .await?;

    kill_and_restart(state.clone()).await?;

    Ok(HttpResponse::Ok().json("block_ip 表已经被重置, ebpf程序也已经重启"))
}
