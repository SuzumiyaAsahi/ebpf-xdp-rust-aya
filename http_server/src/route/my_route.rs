use crate::{block_ip, kill_restart, not_found::http_not_found, package_info};
use actix_web::web;

pub fn route(cfg: &mut web::ServiceConfig) {
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
                "/write_many",
                web::post().to(block_ip::write_block_ip_vec::write_block_ip_vec),
            )
            .route(
                "/delete",
                web::delete().to(block_ip::delete_block_ip::delete_block_ip),
            )
            .route("/flush", web::delete().to(block_ip::flush::flush_all)),
    )
    .service(
        web::scope("/kill_restart")
            .route("/kill", web::get().to(kill_restart::kill::kill_ebpf))
            .route(
                "/restart",
                web::get().to(kill_restart::restart::restart_ebpf),
            )
            .route(
                "/kill_and_restart",
                web::get().to(kill_restart::kill_and_restart::kill_and_restart),
            ),
    )
    .default_service(web::route().to(http_not_found));
}
