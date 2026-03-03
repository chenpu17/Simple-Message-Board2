use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Daemon manager for the message board service
pub struct DaemonManager {
    data_dir: PathBuf,
}

impl DaemonManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Get the PID file path
    pub fn pid_file(&self) -> PathBuf {
        self.data_dir.join("message-board.pid")
    }

    /// Get the log file path
    pub fn log_file(&self) -> PathBuf {
        self.data_dir.join("message-board.log")
    }

    /// Ensure the data directory exists
    pub fn ensure_data_dir(&self) -> io::Result<()> {
        fs::create_dir_all(&self.data_dir)
    }

    /// Read the PID from the PID file
    pub fn read_pid(&self) -> io::Result<Option<u32>> {
        let pid_file = self.pid_file();
        if !pid_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&pid_file)?;
        let pid: u32 = content.trim().parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid PID file content: {}", e),
            )
        })?;

        // Check if process is running
        if self.is_process_running(pid) {
            Ok(Some(pid))
        } else {
            // Clean up stale PID file
            let _ = fs::remove_file(&pid_file);
            Ok(None)
        }
    }

    /// Write PID to the PID file
    pub fn write_pid(&self, pid: u32) -> io::Result<()> {
        self.ensure_data_dir()?;
        let mut file = File::create(self.pid_file())?;
        write!(file, "{}", pid)
    }

    /// Remove the PID file
    pub fn remove_pid(&self) -> io::Result<()> {
        let pid_file = self.pid_file();
        if pid_file.exists() {
            fs::remove_file(pid_file)?;
        }
        Ok(())
    }

    /// Check if a process with the given PID is running
    #[cfg(unix)]
    pub fn is_process_running(&self, pid: u32) -> bool {
        use std::process::Command;
        // Use kill -0 to check if process exists
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    pub fn is_process_running(&self, pid: u32) -> bool {
        use std::process::Command;
        // Use tasklist to check if process exists on Windows
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }

    /// Stop the daemon process
    pub fn stop(&self) -> io::Result<bool> {
        match self.read_pid()? {
            Some(pid) => {
                #[cfg(unix)]
                {
                    // Send SIGTERM
                    let result = Command::new("kill")
                        .arg("-TERM")
                        .arg(pid.to_string())
                        .status();

                    match result {
                        Ok(status) if status.success() => {
                            // Wait for process to terminate
                            for _ in 0..10 {
                                std::thread::sleep(Duration::from_millis(100));
                                if !self.is_process_running(pid) {
                                    break;
                                }
                            }
                            self.remove_pid()?;
                            Ok(true)
                        }
                        _ => {
                            // Force kill if SIGTERM failed
                            let _ = Command::new("kill")
                                .arg("-KILL")
                                .arg(pid.to_string())
                                .status();
                            self.remove_pid()?;
                            Ok(true)
                        }
                    }
                }

                #[cfg(windows)]
                {
                    let result = Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F"])
                        .status();

                    match result {
                        Ok(status) if status.success() => {
                            self.remove_pid()?;
                            Ok(true)
                        }
                        _ => Ok(false),
                    }
                }
            }
            None => Ok(false),
        }
    }

    /// Read the last n lines from the log file
    pub fn read_logs(&self, lines: usize) -> io::Result<Vec<String>> {
        let log_file = self.log_file();
        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&log_file)?;
        let reader = io::BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

        let start = if all_lines.len() > lines {
            all_lines.len() - lines
        } else {
            0
        };

        Ok(all_lines[start..].to_vec())
    }

    /// Append to the log file
    pub fn append_log(&self, message: &str) -> io::Result<()> {
        self.ensure_data_dir()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_file())?;
        writeln!(file, "{}", message)
    }
}

/// Print the status of the daemon
pub fn print_status(daemon: &DaemonManager) {
    match daemon.read_pid() {
        Ok(Some(pid)) => {
            println!("Message board is running");
            println!("PID: {}", pid);
            println!("Data directory: {}", daemon.data_dir.display());
            println!("Log file: {}", daemon.log_file().display());
        }
        Ok(None) => {
            println!("Message board is not running");
            println!("Data directory: {}", daemon.data_dir.display());
        }
        Err(e) => {
            println!("Error checking status: {}", e);
        }
    }
}

/// Print the logs
pub fn print_logs(daemon: &DaemonManager, lines: usize) {
    match daemon.read_logs(lines) {
        Ok(log_lines) => {
            if log_lines.is_empty() {
                println!("No log file found");
                println!("Expected location: {}", daemon.log_file().display());
            } else {
                for line in log_lines {
                    println!("{}", line);
                }
            }
        }
        Err(e) => {
            println!("Error reading logs: {}", e);
        }
    }
}

/// Print the start success message
pub fn print_start_success(pid: u32, port: u16, data_dir: &Path, log_file: &Path) {
    println!("Message board started successfully!");
    println!("PID: {}", pid);
    println!("Port: {}", port);
    println!("Data directory: {}", data_dir.display());
    println!("Log file: {}", log_file.display());
    println!();
    println!("Access at http://localhost:{}", port);
    println!();
    println!("Use 'message-board stop' to stop the service");
    println!("Use 'message-board logs' to view logs");
}

/// Print the stop success message
pub fn print_stop_success(pid: u32) {
    println!("Message board stopped (PID: {})", pid);
}

/// Print the already running message
pub fn print_already_running(pid: u32, port: u16) {
    println!("Message board is already running (PID: {})", pid);
    println!("Access at http://localhost:{}", port);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_daemon_manager_pid_file() {
        let dir = tempdir().unwrap();
        let daemon = DaemonManager::new(dir.path().to_path_buf());
        assert!(daemon.pid_file().ends_with("message-board.pid"));
    }

    #[test]
    fn test_daemon_manager_log_file() {
        let dir = tempdir().unwrap();
        let daemon = DaemonManager::new(dir.path().to_path_buf());
        assert!(daemon.log_file().ends_with("message-board.log"));
    }

    #[test]
    fn test_ensure_data_dir() {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("test_subdir");
        let daemon = DaemonManager::new(data_dir.clone());
        assert!(daemon.ensure_data_dir().is_ok());
        assert!(data_dir.exists());
    }

    #[test]
    fn test_write_and_read_pid() {
        let dir = tempdir().unwrap();
        let daemon = DaemonManager::new(dir.path().to_path_buf());
        daemon.ensure_data_dir().unwrap();

        assert!(daemon.write_pid(12345).is_ok());
        // Note: read_pid will return None because process 12345 doesn't exist
        // and it cleans up stale PID files
        let result = daemon.read_pid().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_append_and_read_logs() {
        let dir = tempdir().unwrap();
        let daemon = DaemonManager::new(dir.path().to_path_buf());
        daemon.ensure_data_dir().unwrap();

        assert!(daemon.append_log("Line 1").is_ok());
        assert!(daemon.append_log("Line 2").is_ok());
        assert!(daemon.append_log("Line 3").is_ok());

        let logs = daemon.read_logs(2).unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], "Line 2");
        assert_eq!(logs[1], "Line 3");
    }

    #[test]
    fn test_read_logs_empty() {
        let dir = tempdir().unwrap();
        let daemon = DaemonManager::new(dir.path().to_path_buf());

        let logs = daemon.read_logs(10).unwrap();
        assert!(logs.is_empty());
    }

    #[test]
    fn test_remove_pid() {
        let dir = tempdir().unwrap();
        let daemon = DaemonManager::new(dir.path().to_path_buf());
        daemon.ensure_data_dir().unwrap();

        daemon.write_pid(12345).unwrap();
        assert!(daemon.pid_file().exists());

        assert!(daemon.remove_pid().is_ok());
        assert!(!daemon.pid_file().exists());
    }
}
