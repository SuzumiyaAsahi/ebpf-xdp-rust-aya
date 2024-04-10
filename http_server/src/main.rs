use actix_web::{middleware, web, App, HttpServer};
use env_logger::Env;
use errors::MyError;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::{env, io, sync::Arc};

#[path = "./errors.rs"]
mod errors;

#[path = "./package_info/mod.rs"]
mod package_info;

#[path = "./block_ip/mod.rs"]
mod block_ip;

#[path = "./kill_restart/mod.rs"]
mod kill_restart;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: Pool<Sqlite>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let db_url = env::var("DATABASE_URL").expect("Please set 'DATABASE_URL'");
    let app_state = Arc::new(AppState {
        db_pool: SqlitePool::connect(&db_url).await.unwrap(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&app_state)))
            .wrap(middleware::Logger::default())
            .configure(route)
    })
    .bind("127.0.0.1:12345")?
    .run()
    .await
}

fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/package_info")
            .route("/read", web::get().to(package_info::view::get_package_info)),
    )
    .service(
        web::scope("/blocked_ip")
            .route(
                "/read",
                web::get().to(block_ip::read_block_ip::read_block_ip),
            )
            .route(
                "/write",
                web::post().to(block_ip::write_block_ip::write_block_ip),
            )
            .route(
                "/delete",
                web::delete().to(block_ip::delete_block_ip::delete_block_ip),
            ),
    )
    .service(
        web::scope("/kill_restart").route("/kill", web::delete().to(kill_restart::kill::kill_ebpf)),
    );
}
