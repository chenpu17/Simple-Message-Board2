use actix_files as fs;
use actix_web::{web, App, HttpServer};
use message_board::db::Repository;
use message_board::handlers;
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // 初始化数据库
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/messages.db?mode=rwc".to_string());

    // 端口配置：支持环境变量，默认为13478（与原Node.js版本兼容）
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(13478);

    // 确保数据目录存在
    if let Err(e) = std::fs::create_dir_all("data") {
        tracing::error!("Failed to create data directory: {}", e);
    }

    let repo = match Repository::new(&database_url).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Server starting at http://127.0.0.1:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(repo.clone()))
            .route("/", web::get().to(handlers::home))
            .route("/dashboard", web::get().to(handlers::dashboard))
            .route("/submit", web::post().to(handlers::submit_message))
            .route("/delete", web::post().to(handlers::delete_message))
            .route("/reply", web::post().to(handlers::submit_reply))
            .route("/delete-reply", web::post().to(handlers::delete_reply))
            .route("/api/messages", web::get().to(handlers::api_messages))
            .route("/api/tags", web::get().to(handlers::api_tags))
            .service(fs::Files::new("/static", "./public"))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
