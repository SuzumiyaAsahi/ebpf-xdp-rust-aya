use crate::{AppState, MyError};
use actix_web::{web, HttpResponse};
use std::{process::Command, sync::Arc, thread, time::Duration};

pub async fn kill_ebpf(_state: web::Data<Arc<AppState>>) -> Result<HttpResponse, MyError> {
    let s = sysinfo::System::new_all();

    let target_name_1 = "test-app";
    let processes = s.processes_by_name(target_name_1);

    for process in processes {
        Command::new("kill")
            .arg(format!("{}", process.pid()))
            .output()?;
        thread::sleep(Duration::from_millis(1000));
    }

    Ok(HttpResponse::Ok().json("ebpf程序已经关闭".to_string()))
}
