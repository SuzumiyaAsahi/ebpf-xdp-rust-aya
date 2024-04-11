use crate::{AppState, MyError};
use actix_web::{web, HttpResponse};
use std::{
    process::{Command, Stdio},
    sync::Arc,
};

pub async fn restart_ebpf(_state: web::Data<Arc<AppState>>) -> Result<HttpResponse, MyError> {
    Command::new("make")
        .current_dir("../")
        .env("DATABASE_URL", "sqlite://./identifier.sqlite")
        // .stdin(Stdio::null())
        // .stderr(Stdio::null())
        // .stdout(Stdio::null())
        .arg("ebpf")
        .spawn()?;

    Ok(HttpResponse::Ok().json("ebpf程序重新启动".to_string()))
}
