use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Get the default data directory (~/.message-board)
fn get_default_data_dir() -> PathBuf {
    // Priority: DATA_DIR env var > ~/.message-board
    if let Ok(data_dir) = std::env::var("DATA_DIR") {
        return PathBuf::from(data_dir);
    }

    // Use home directory (compatible with Node.js version)
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".message-board")
}

/// Get the default port
fn get_default_port() -> u16 {
    // Priority: PORT env var > 13478
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(13478)
}

/// Message Board CLI - 简易留言板命令行工具
#[derive(Parser, Debug)]
#[command(name = "message-board")]
#[command(author = "chenpu17")]
#[command(version)]
#[command(about = "A simple message board server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Port to listen on (default: 13478, can also set PORT env var)
    #[arg(short, long, global = true)]
    pub port: Option<u16>,

    /// Data directory for database and logs (default: ~/.message-board, can also set DATA_DIR env var)
    #[arg(short = 'd', long, global = true, value_name = "DIR")]
    pub data_dir: Option<PathBuf>,

    /// Run in foreground (not as daemon)
    #[arg(short, long, global = true)]
    pub foreground: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Start the message board service (default, runs as daemon)
    Start {
        /// Port to listen on (default: 13478, can also set PORT env var)
        #[arg(short, long)]
        port: Option<u16>,

        /// Data directory for database and logs (default: ~/.message-board, can also set DATA_DIR env var)
        #[arg(short = 'd', long, value_name = "DIR")]
        data_dir: Option<PathBuf>,

        /// Run in foreground (not as daemon)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Stop the message board service
    Stop {
        /// Data directory for database and logs (default: ~/.message-board, can also set DATA_DIR env var)
        #[arg(short = 'd', long, value_name = "DIR")]
        data_dir: Option<PathBuf>,
    },

    /// Restart the message board service
    Restart {
        /// Port to listen on (default: 13478, can also set PORT env var)
        #[arg(short, long)]
        port: Option<u16>,

        /// Data directory for database and logs (default: ~/.message-board, can also set DATA_DIR env var)
        #[arg(short = 'd', long, value_name = "DIR")]
        data_dir: Option<PathBuf>,

        /// Run in foreground (not as daemon)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Show service status
    Status {
        /// Data directory for database and logs (default: ~/.message-board, can also set DATA_DIR env var)
        #[arg(short = 'd', long, value_name = "DIR")]
        data_dir: Option<PathBuf>,
    },

    /// Show recent logs
    Logs {
        /// Data directory for database and logs (default: ~/.message-board, can also set DATA_DIR env var)
        #[arg(short = 'd', long, value_name = "DIR")]
        data_dir: Option<PathBuf>,

        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Get the effective command (defaults to Start if none specified)
    pub fn get_command(&self) -> Commands {
        self.command.clone().unwrap_or(Commands::Start {
            port: self.port,
            data_dir: self.data_dir.clone(),
            foreground: self.foreground,
        })
    }

    /// Get effective port (CLI arg > PORT env var > default 13478)
    pub fn get_port(&self) -> u16 {
        let default_port = get_default_port();
        match &self.command {
            Some(Commands::Start { port, .. }) => port.or(Some(default_port)),
            Some(Commands::Restart { port, .. }) => port.or(Some(default_port)),
            _ => self.port,
        }
        .unwrap_or(default_port)
    }

    /// Get effective data directory (CLI arg > DATA_DIR env var > ~/.message-board)
    pub fn get_data_dir(&self) -> PathBuf {
        let default_dir = get_default_data_dir();
        let dir = match &self.command {
            Some(Commands::Start { data_dir, .. }) => data_dir.clone(),
            Some(Commands::Stop { data_dir }) => data_dir.clone(),
            Some(Commands::Restart { data_dir, .. }) => data_dir.clone(),
            Some(Commands::Status { data_dir }) => data_dir.clone(),
            Some(Commands::Logs { data_dir, .. }) => data_dir.clone(),
            None => self.data_dir.clone(),
        };

        dir.unwrap_or(default_dir)
    }

    /// Get effective foreground flag
    pub fn get_foreground(&self) -> bool {
        match &self.command {
            Some(Commands::Start { foreground, .. }) => *foreground,
            Some(Commands::Restart { foreground, .. }) => *foreground,
            _ => self.foreground,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_version_flag() {
        // --version causes clap to print version and exit
        // try_parse_from returns Err with DisplayVersion kind
        let result = Cli::try_parse_from(["message-board", "--version"]);
        // clap 4 returns an error with kind DisplayVersion for --version
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_help_flag() {
        // --help causes clap to print help and exit
        // try_parse_from returns Err with DisplayHelp kind
        let result = Cli::try_parse_from(["message-board", "--help"]);
        // clap 4 returns an error with kind DisplayHelp for --help
        assert!(result.is_err());
    }

    #[test]
    fn test_start_command_default() {
        let cli = Cli::try_parse_from(["message-board", "start"]).unwrap();
        match cli.get_command() {
            Commands::Start { port, .. } => assert!(port.is_none()), // Uses env/default
            _ => panic!("Expected Start command"),
        }
        // get_port() should return the default
        assert_eq!(cli.get_port(), 13478);
    }

    #[test]
    fn test_start_command_with_port() {
        let cli = Cli::try_parse_from(["message-board", "start", "-p", "8080"]).unwrap();
        match cli.get_command() {
            Commands::Start { port, .. } => assert_eq!(port, Some(8080)),
            _ => panic!("Expected Start command"),
        }
        assert_eq!(cli.get_port(), 8080);
    }

    #[test]
    fn test_start_command_with_data_dir() {
        let cli = Cli::try_parse_from(["message-board", "start", "-d", "/tmp/test"]).unwrap();
        match cli.get_command() {
            Commands::Start { data_dir, .. } => {
                assert_eq!(data_dir, Some(PathBuf::from("/tmp/test")));
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_start_command_foreground() {
        let cli = Cli::try_parse_from(["message-board", "start", "-f"]).unwrap();
        match cli.get_command() {
            Commands::Start { foreground, .. } => assert!(foreground),
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_stop_command() {
        let cli = Cli::try_parse_from(["message-board", "stop"]).unwrap();
        match cli.get_command() {
            Commands::Stop { .. } => {}
            _ => panic!("Expected Stop command"),
        }
    }

    #[test]
    fn test_restart_command() {
        let cli = Cli::try_parse_from(["message-board", "restart"]).unwrap();
        match cli.get_command() {
            Commands::Restart { .. } => {}
            _ => panic!("Expected Restart command"),
        }
    }

    #[test]
    fn test_status_command() {
        let cli = Cli::try_parse_from(["message-board", "status"]).unwrap();
        match cli.get_command() {
            Commands::Status { .. } => {}
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_logs_command() {
        let cli = Cli::try_parse_from(["message-board", "logs"]).unwrap();
        match cli.get_command() {
            Commands::Logs { lines, .. } => assert_eq!(lines, 50),
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn test_logs_command_with_lines() {
        let cli = Cli::try_parse_from(["message-board", "logs", "--lines", "100"]).unwrap();
        match cli.get_command() {
            Commands::Logs { lines, .. } => assert_eq!(lines, 100),
            _ => panic!("Expected Logs command"),
        }
    }

    #[test]
    fn test_default_data_dir_uses_home() {
        let dir = get_default_data_dir();
        // Should end with .message-board and be in home directory
        assert!(dir.ends_with(".message-board"));
    }

    #[test]
    fn test_default_port() {
        let port = get_default_port();
        assert_eq!(port, 13478);
    }
}
