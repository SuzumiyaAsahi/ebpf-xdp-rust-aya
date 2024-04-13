use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use env_logger::Env;
use errors::MyError;
use route::my_route;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::{env, io, sync::Arc};

#[path = "./errors.rs"]
mod errors;

#[path = "./not_found.rs"]
mod not_found;

#[path = "./package_info/mod.rs"]
mod package_info;

#[path = "./block_ip/mod.rs"]
mod block_ip;

#[path = "./kill_restart/mod.rs"]
mod kill_restart;

#[path = "./route/mod.rs"]
mod route;

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
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(Arc::clone(&app_state)))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .configure(my_route::route)
    })
    .bind("127.0.0.1:12345")?
    .run()
    .await
}
