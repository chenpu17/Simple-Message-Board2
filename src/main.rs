use actix_files as fs;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use message_board::cli::{Cli, Commands};
use message_board::config;
use message_board::daemon::{
    print_already_running, print_logs, print_start_failure, print_start_success, print_status,
    print_stop_success, DaemonManager,
};
use message_board::db::Repository;
use message_board::handlers;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing_subscriber::EnvFilter;

/// Get the current executable path
fn get_current_exe() -> std::io::Result<PathBuf> {
    std::env::current_exe()
}

/// Determine the static files directory
/// Priority:
/// 1. ./public (current working directory) - for development and npm package
/// 2. <executable_dir>/public - for standalone binary
/// 3. ./public as fallback (will show error if not found)
fn get_static_dir() -> PathBuf {
    // First check current working directory (for npm package and development)
    let cwd_public = PathBuf::from("./public");
    if cwd_public.exists() {
        return cwd_public;
    }

    // Check relative to executable (for standalone binary)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let exe_public = exe_dir.join("public");
            if exe_public.exists() {
                return exe_public;
            }
        }
    }

    // Fallback to cwd (will show error if not found, but at least won't panic)
    cwd_public
}

/// Run the server (called from daemon or foreground mode)
async fn run_server(port: u16, data_dir: PathBuf) -> std::io::Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // 初始化数据库
    // Priority: DATABASE_URL env > data_dir/messages.db
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| format!("sqlite:{}?mode=rwc", data_dir.join("messages.db").display()));

    // 确保数据目录存在
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        tracing::error!("Failed to create data directory: {}", e);
    }

    let repo = match Repository::new(&database_url).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Get static files directory
    let static_dir = get_static_dir();
    tracing::info!("Static files directory: {}", static_dir.display());

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
            .service(fs::Files::new("/static", static_dir.clone()))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

/// Start the daemon (spawn background process)
fn start_daemon(port: u16, data_dir: PathBuf, daemon: &DaemonManager) -> std::io::Result<()> {
    // Check if already running
    if let Some(pid) = daemon.read_pid()? {
        print_already_running(pid, port);
        return Ok(());
    }

    daemon.ensure_data_dir()?;

    // Get current executable
    let exe = get_current_exe()?;

    #[cfg(unix)]
    {
        // Start daemon process
        let log_file = daemon.log_file();
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        // Build the command with arguments
        let mut cmd = Command::new(&exe);
        cmd.arg("start")
            .arg("--port")
            .arg(port.to_string())
            .arg("--data-dir")
            .arg(&data_dir)
            .arg("--foreground")
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log);

        // Pass through DATABASE_URL if set, otherwise compute from data_dir
        // This ensures consistent behavior between foreground and daemon modes
        if std::env::var("DATABASE_URL").is_ok() {
            // Pass through existing DATABASE_URL
            for (key, value) in std::env::vars() {
                if key == "DATABASE_URL" || key == "PORT" || key == "DATA_DIR" {
                    cmd.env(key, value);
                }
            }
        } else {
            // Set DATABASE_URL based on data_dir
            cmd.env(
                "DATABASE_URL",
                format!("sqlite:{}?mode=rwc", data_dir.join("messages.db").display()),
            );
        }

        let child = cmd.spawn()?;

        let pid = child.id();

        // Wait for the process to be ready (listening on port)
        // Timeout after 5 seconds
        let ready = daemon.wait_for_process_ready(pid, port, 5000);

        if ready {
            daemon.write_pid(pid)?;
            print_start_success(pid, port, &data_dir, &daemon.log_file());
        } else {
            // Process failed to start
            // Check if process is still running
            if daemon.is_process_running(pid) {
                // Process is running but not responding on port - might be slow
                // Still write PID and report success with warning
                daemon.write_pid(pid)?;
                println!(
                    "Warning: Process started but not responding on port {} yet.",
                    port
                );
                println!(
                    "Check the log file for details: {}",
                    daemon.log_file().display()
                );
                print_start_success(pid, port, &data_dir, &daemon.log_file());
            } else {
                // Process exited
                print_start_failure(&daemon.log_file());
                return Err(std::io::Error::other("Process failed to start"));
            }
        }
    }

    #[cfg(windows)]
    {
        // Windows doesn't support true daemon mode well, run in foreground with a note
        println!("Note: Daemon mode on Windows runs in the current console.");
        println!("Use Ctrl+C to stop the service.");
        println!();
        run_server_blocking(port, data_dir)?;
    }

    Ok(())
}

/// Run the server in blocking mode
fn run_server_blocking(port: u16, data_dir: PathBuf) -> std::io::Result<()> {
    // Use tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_server(port, data_dir))
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let command = cli.get_command();
    let port = cli.get_port();
    let data_dir = cli.get_data_dir();
    let foreground = cli.get_foreground();

    let daemon = DaemonManager::new(data_dir.clone());

    match command {
        Commands::Start { .. } => {
            if foreground {
                // Run in foreground
                println!("Starting message board on port {}...", port);
                println!("Data directory: {}", data_dir.display());
                println!("Access at http://localhost:{}", port);
                println!();
                run_server_blocking(port, data_dir)?;
            } else {
                // Run as daemon
                start_daemon(port, data_dir, &daemon)?;
            }
        }
        Commands::Stop { .. } => match daemon.read_pid()? {
            Some(pid) => {
                if daemon.stop()? {
                    print_stop_success(pid);
                } else {
                    println!("Failed to stop message board");
                    std::process::exit(1);
                }
            }
            None => {
                println!("Message board is not running");
            }
        },
        Commands::Restart { .. } => {
            // Stop existing instance
            if let Some(pid) = daemon.read_pid()? {
                println!("Stopping existing instance...");
                if daemon.stop()? {
                    println!("Stopped (PID: {})", pid);
                }
                // Wait for graceful shutdown
                std::thread::sleep(std::time::Duration::from_secs(1));
            }

            // Start new instance
            println!("Starting new instance...");
            if foreground {
                println!("Starting message board on port {}...", port);
                println!("Data directory: {}", data_dir.display());
                println!("Access at http://localhost:{}", port);
                println!();
                run_server_blocking(port, data_dir)?;
            } else {
                start_daemon(port, data_dir, &daemon)?;
            }
        }
        Commands::Status { .. } => {
            print_status(&daemon);
        }
        Commands::Logs { lines, .. } => {
            print_logs(&daemon, lines);
        }
        Commands::Version => {
            println!("simple-message-board v{}", config::VERSION);
        }
    }

    Ok(())
}
