use crate::{
    kill_restart::{kill, restart},
    AppState, MyError,
};
use actix_web::{web, HttpResponse};
use std::sync::Arc;

pub async fn kill_and_restart(state: web::Data<Arc<AppState>>) -> Result<HttpResponse, MyError> {
    kill::kill_ebpf(state.clone()).await?;
    restart::restart_ebpf(state.clone()).await?;
    Ok(HttpResponse::Ok().json("ebpf程序已经被关闭后再次重启"))
}
