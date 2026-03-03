use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Process identifier with verification info
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub command: String,
}

impl ProcessInfo {
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            command: "message-board".to_string(),
        }
    }
}

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
        self.read_pid_info().map(|opt| opt.map(|info| info.pid))
    }

    /// Read the PID info from the PID file with verification
    pub fn read_pid_info(&self) -> io::Result<Option<ProcessInfo>> {
        let pid_file = self.pid_file();
        if !pid_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&pid_file)?;
        let parts: Vec<&str> = content.trim().split(':').collect();

        if parts.is_empty() {
            return Ok(None);
        }

        let pid: u32 = parts[0].parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid PID file content: {}", e),
            )
        })?;

        let expected_command = parts.get(1).unwrap_or(&"message-board").to_string();

        // Check if process is running AND verify it's our process
        if self.is_process_running(pid)
            && self.verify_process_is_message_board(pid, &expected_command)
        {
            Ok(Some(ProcessInfo {
                pid,
                command: expected_command,
            }))
        } else {
            // Clean up stale PID file
            let _ = fs::remove_file(&pid_file);
            Ok(None)
        }
    }

    /// Write PID to the PID file
    pub fn write_pid(&self, pid: u32) -> io::Result<()> {
        self.write_pid_info(&ProcessInfo::new(pid))
    }

    /// Write PID info to the PID file
    pub fn write_pid_info(&self, info: &ProcessInfo) -> io::Result<()> {
        self.ensure_data_dir()?;
        let mut file = File::create(self.pid_file())?;
        write!(file, "{}:{}", info.pid, info.command)
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
        // Use kill -0 to check if process exists (suppress stderr)
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    pub fn is_process_running(&self, pid: u32) -> bool {
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

    /// Verify that the process with the given PID is actually message-board
    #[cfg(unix)]
    pub fn verify_process_is_message_board(&self, pid: u32, expected_command: &str) -> bool {
        // Read /proc/<pid>/comm or /proc/<pid>/cmdline to verify process name
        let comm_path = format!("/proc/{}/comm", pid);
        if let Ok(comm) = fs::read_to_string(&comm_path) {
            let comm = comm.trim();
            // Check if the process name contains "message-board" or matches expected
            return comm.contains("message_board") || comm == expected_command;
        }

        // Fallback: try /proc/<pid>/cmdline
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
            // cmdline uses null bytes as separators
            let cmdline = cmdline.replace('\0', " ");
            return cmdline.contains("message_board") || cmdline.contains("message-board");
        }

        // macOS doesn't have /proc, use ps command
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output();

        if let Ok(output) = output {
            let name = String::from_utf8_lossy(&output.stdout);
            let name = name.trim();
            return name.contains("message_board") || name == expected_command;
        }

        // If we can't verify, assume it's not our process (safer)
        false
    }

    #[cfg(windows)]
    pub fn verify_process_is_message_board(&self, pid: u32, expected_command: &str) -> bool {
        // Use wmic to get process name
        let output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("ProcessId={}", pid),
                "get",
                "Name",
                "/value",
            ])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check for message_board.exe or expected command
            return stdout.contains("message_board")
                || stdout.contains(expected_command)
                || stdout.contains("message-board");
        }

        // If we can't verify, assume it's not our process (safer)
        false
    }

    /// Stop the daemon process
    pub fn stop(&self) -> io::Result<bool> {
        match self.read_pid_info()? {
            Some(info) => {
                let pid = info.pid;

                #[cfg(unix)]
                {
                    // Verify again before stopping
                    if !self.verify_process_is_message_board(pid, &info.command) {
                        // Not our process, clean up PID file but don't kill
                        self.remove_pid()?;
                        return Ok(false);
                    }

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
                    // Verify again before stopping
                    if !self.verify_process_is_message_board(pid, &info.command) {
                        self.remove_pid()?;
                        return Ok(false);
                    }

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

    /// Wait for the process to start and verify it's listening
    pub fn wait_for_process_ready(&self, pid: u32, host: &str, port: u16, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();

        // When binding to 0.0.0.0 or ::, connect to localhost for testing
        let connect_host = if host == "0.0.0.0" {
            "127.0.0.1"
        } else if host == "::" {
            "::1"
        } else {
            host
        };

        // Format address with proper IPv6 handling
        let addr = if connect_host.contains(':') && !connect_host.starts_with('[') {
            format!("[{}]:{}", connect_host, port)
        } else {
            format!("{}:{}", connect_host, port)
        };

        while start.elapsed().as_millis() < timeout_ms as u128 {
            // Check if process is still running
            if !self.is_process_running(pid) {
                return false;
            }

            // Try to connect to the port
            if std::net::TcpStream::connect(&addr).is_ok() {
                return true;
            }

            std::thread::sleep(Duration::from_millis(50));
        }

        false
    }
}

/// Print the status of the daemon
pub fn print_status(daemon: &DaemonManager) {
    match daemon.read_pid_info() {
        Ok(Some(info)) => {
            println!("Message board is running");
            println!("PID: {}", info.pid);
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
pub fn print_start_success(pid: u32, host: &str, port: u16, data_dir: &Path, log_file: &Path) {
    println!("Message board started successfully!");
    println!("PID: {}", pid);
    println!("Host: {}", host);
    println!("Port: {}", port);
    println!("Data directory: {}", data_dir.display());
    println!("Log file: {}", log_file.display());
    println!();
    println!("Access at http://{}:{}", host, port);
    println!();
    println!("Use 'message-board stop' to stop the service");
    println!("Use 'message-board logs' to view logs");
}

/// Print the stop success message
pub fn print_stop_success(pid: u32) {
    println!("Message board stopped (PID: {})", pid);
}

/// Print the already running message
pub fn print_already_running(pid: u32, host: &str, port: u16) {
    println!("Message board is already running (PID: {})", pid);
    println!("Access at http://{}:{}", host, port);
}

/// Print the start failure message
pub fn print_start_failure(log_file: &Path) {
    println!("Failed to start message board!");
    println!("Check the log file for details: {}", log_file.display());
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
    fn test_write_and_read_pid_info() {
        let dir = tempdir().unwrap();
        let daemon = DaemonManager::new(dir.path().to_path_buf());
        daemon.ensure_data_dir().unwrap();

        let info = ProcessInfo::new(12345);
        assert!(daemon.write_pid_info(&info).is_ok());

        // Check file content format
        let content = fs::read_to_string(daemon.pid_file()).unwrap();
        assert!(content.contains("12345"));
        assert!(content.contains("message-board"));
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
