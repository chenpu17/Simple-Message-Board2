//! 静态文件服务测试
//!
//! 测试 /static/* 路由的静态文件服务功能

use actix_web::{test, App};
use actix_files as fs;

/// 测试静态文件服务 - 不存在的文件
#[actix_rt::test]
async fn test_static_file_not_found() {
    let app = test::init_service(
        App::new()
            .service(fs::Files::new("/static", "./public"))
    )
    .await;

    // 请求不存在的文件
    let req = test::TestRequest::get()
        .uri("/static/nonexistent.css")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // 应该返回 404
    assert_eq!(resp.status().as_u16(), 404);
}

/// 测试静态文件服务 - 路径遍历攻击防护
#[actix_rt::test]
async fn test_static_file_path_traversal_protection() {
    let app = test::init_service(
        App::new()
            .service(fs::Files::new("/static", "./public"))
    )
    .await;

    // 尝试路径遍历攻击
    let path_traversal_attempts = vec![
        "/static/../Cargo.toml",
        "/static/..%2FCargo.toml",
        "/static/....//....//Cargo.toml",
        "/static/..%252f..%252fCargo.toml",
    ];

    for attempt in path_traversal_attempts {
        let req = test::TestRequest::get()
            .uri(attempt)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // 路径遍历应该被阻止（返回 404 或 400）
        let status = resp.status().as_u16();
        assert!(
            status == 404 || status == 400 || status == 403,
            "Path traversal attempt '{}' should be blocked, got status {}",
            attempt, status
        );
    }
}

/// 测试静态文件服务 - 存在的文件
#[actix_rt::test]
async fn test_static_file_exists() {
    // 检查 public 目录下是否有文件
    let public_dir = std::path::Path::new("./public");
    if !public_dir.exists() {
        // 如果 public 目录不存在，跳过此测试
        eprintln!("Skipping test: public directory does not exist");
        return;
    }

    let app = test::init_service(
        App::new()
            .service(fs::Files::new("/static", "./public"))
    )
    .await;

    // 尝试获取存在的文件
    // 检查 app.css 是否存在
    let app_css = public_dir.join("app.css");
    if app_css.exists() {
        let req = test::TestRequest::get()
            .uri("/static/app.css")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // 应该返回 200
        assert!(resp.status().is_success(), "app.css should be served");

        // 验证 Content-Type
        let content_type = resp.headers().get("content-type");
        assert!(content_type.is_some());
        let ct = content_type.unwrap().to_str().unwrap();
        assert!(ct.contains("text/css") || ct.contains("text/plain"));
    }

    // 检查 app.js 是否存在
    let app_js = public_dir.join("app.js");
    if app_js.exists() {
        let req = test::TestRequest::get()
            .uri("/static/app.js")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // 应该返回 200
        assert!(resp.status().is_success(), "app.js should be served");
    }
}

/// 测试静态文件服务 - 子目录访问
#[actix_rt::test]
async fn test_static_file_subdirectory() {
    let app = test::init_service(
        App::new()
            .service(fs::Files::new("/static", "./public"))
    )
    .await;

    // 尝试访问子目录（如果存在）
    let req = test::TestRequest::get()
        .uri("/static/themes/")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // 目录访问的行为取决于配置，可能返回 404 或 403
    let status = resp.status().as_u16();
    // actix-files 默认不列出目录内容
    assert!(status == 404 || status == 403 || status == 301 || status == 200);
}

/// 测试静态文件服务 - 空路径
#[actix_rt::test]
async fn test_static_file_empty_path() {
    let app = test::init_service(
        App::new()
            .service(fs::Files::new("/static", "./public"))
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/static")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // 空路径可能返回 404 或重定向
    let status = resp.status().as_u16();
    assert!(status == 404 || status == 301 || status == 302 || status == 200);
}

/// 测试静态文件服务 - 特殊字符文件名
#[actix_rt::test]
async fn test_static_file_special_characters() {
    let app = test::init_service(
        App::new()
            .service(fs::Files::new("/static", "./public"))
    )
    .await;

    // 测试 URL 编码的文件名
    let special_paths = vec![
        "/static/file%20with%20spaces.css",
        "/static/file%2Bplus.css",
    ];

    for path in special_paths {
        let req = test::TestRequest::get()
            .uri(path)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // 这些文件不存在，应该返回 404
        // 主要测试的是不会因为特殊字符而崩溃
        let status = resp.status().as_u16();
        assert!(status == 404 || status == 400 || status == 200);
    }
}
