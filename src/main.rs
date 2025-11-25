use chrono::{DateTime, Local};
use notify_rust::Notification;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::Duration as StdDuration;
mod pomodoro;

pub const POLL_INTERVAL_MS: u64 = 1000; // Check active window every second
#[derive(Debug, Deserialize)]
struct HyprlandWindow {
    title: String,
}

#[derive(Debug)]
struct DomainTracker {
    time_spent: HashMap<String, i64>, // domain -> seconds
    current_domain: Option<String>,
    session_start: DateTime<Local>,
    mode: pomodoro::pomodoro::PomodoroMode,
    mode_start: DateTime<Local>,
    log_file: Option<String>,
}

impl DomainTracker {
    fn new(log_file: Option<String>) -> Self {
        let now = Local::now();
        if let Some(ref path) = log_file {
            let _ = Self::log_to_file(
                path,
                &format!(
                    "=== Session started at {} ===",
                    now.format("%Y-%m-%d %H:%M:%S")
                ),
            );
        }
        Self {
            time_spent: HashMap::new(),
            current_domain: None,
            session_start: now,
            mode: pomodoro::pomodoro::PomodoroMode::Work,
            mode_start: now,
            log_file,
        }
    }

    fn log_to_file(path: &str, message: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", message)?;
        Ok(())
    }

    fn log(&self, message: &str) {
        if let Some(ref path) = self.log_file {
            let _ = Self::log_to_file(path, message);
        }
    }

    fn get_mode_duration(&self) -> i64 {
        (Local::now() - self.mode_start).num_seconds()
    }

    fn get_mode_remaining(&self) -> i64 {
        let target = match self.mode {
            pomodoro::pomodoro::PomodoroMode::Work => pomodoro::pomodoro::POMODORO_WORK_MINUTES,
            pomodoro::pomodoro::PomodoroMode::Break => pomodoro::pomodoro::POMODORO_BREAK_MINUTES,
        };
        (target * 60) - self.get_mode_duration()
    }

    fn switch_mode(&mut self) {
        self.mode = match self.mode {
            pomodoro::pomodoro::PomodoroMode::Work => pomodoro::pomodoro::PomodoroMode::Break,
            pomodoro::pomodoro::PomodoroMode::Break => pomodoro::pomodoro::PomodoroMode::Work,
        };
        self.mode_start = Local::now();
        let msg = format!(
            "[{}] Switched to {} mode",
            Local::now().format("%H:%M:%S"),
            self.mode.as_str()
        );
        println!("\n{} {}", self.mode.emoji(), msg);
        self.log(&msg);
    }

    fn update(&mut self, domain: Option<String>) {
        if let Some(ref current) = self.current_domain {
            *self.time_spent.entry(current.clone()).or_insert(0) += 1;
        }
        self.current_domain = domain;
    }

    fn get_session_duration(&self) -> i64 {
        (Local::now() - self.session_start).num_seconds()
    }

    fn should_switch_mode(&mut self) -> bool {
        let mode_minutes = self.get_mode_duration() / 60;
        let target_minutes = match self.mode {
            pomodoro::pomodoro::PomodoroMode::Work => pomodoro::pomodoro::POMODORO_WORK_MINUTES,
            pomodoro::pomodoro::PomodoroMode::Break => pomodoro::pomodoro::POMODORO_BREAK_MINUTES,
        };

        mode_minutes >= target_minutes
    }

    fn print_stats(&self) {
        println!("\n--- Session Statistics ---");
        println!(
            "Session duration: {} minutes",
            self.get_session_duration() / 60
        );
        println!("\nTime spent per domain:");

        let mut sorted: Vec<_> = self.time_spent.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        for (domain, seconds) in sorted {
            let minutes = seconds / 60;
            let secs = seconds % 60;
            println!("  {} - {}m {}s", domain, minutes, secs);
        }
        println!("------------------------\n");
    }
}

fn get_active_window_title() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()?;

    if !output.status.success() {
        return Ok(String::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.trim().is_empty() {
        return Ok(String::new());
    }

    let window: HyprlandWindow = serde_json::from_str(&stdout)?;
    println!("{:?}", window);
    Ok(window.title)
}

fn extract_domain_from_title(title: &str) -> Option<String> {
    // Common TLDs to look for
    let tlds = [
        "com", "org", "net", "io", "dev", "co", "ai", "app", "tech", "cloud", "edu", "gov", "mil",
        "int", "info", "biz", "name", "museum", "uk", "us", "ca", "au", "de", "fr", "it", "es",
        "nl", "jp", "cn", "in", "br",
    ];

    let tld_pattern = tlds.join("|");

    // Try multiple patterns in order of specificity
    let patterns = vec![
        // Full URL with protocol (https://github.com/user/repo)
        format!(
            r"https?://(?:www\.)?([a-zA-Z0-9-]+\.(?:{}))(?:/|$|\s)",
            tld_pattern
        ),
        // Domain with www prefix
        format!(r"\bwww\.([a-zA-Z0-9-]+\.(?:{}))(?:/|$|\s|\))", tld_pattern),
        // Standalone domain (github.com, docs.rs, etc.)
        format!(r"\b([a-zA-Z0-9-]+\.(?:{}))(?:/|$|\s|\)|:)", tld_pattern),
        // Domain at start of title
        format!(r"^([a-zA-Z0-9-]+\.(?:{}))", tld_pattern),
    ];

    for pattern_str in patterns {
        if let Ok(pattern) = Regex::new(&pattern_str) {
            if let Some(captures) = pattern.captures(title) {
                if let Some(domain) = captures.get(1) {
                    let domain_str = domain.as_str().to_lowercase();
                    // Filter out overly generic domains
                    if !domain_str.starts_with("www.") {
                        return Some(domain_str);
                    }
                }
            }
        }
    }

    // Fallback: Check for common service names and map to domains
    let title_lower = title.to_lowercase();
    if title_lower.contains("youtube") {
        return Some("youtube.com".to_string());
    } else if title_lower.contains("reddit") {
        return Some("reddit.com".to_string());
    } else if title_lower.contains("twitter") {
        return Some("twitter.com".to_string());
    } else if title_lower.contains("github") {
        return Some("github.com".to_string());
    } else if title_lower.contains("gitlab") {
        return Some("gitlab.com".to_string());
    } else if title_lower.contains("stackoverflow") || title_lower.contains("stack overflow") {
        return Some("stackoverflow.com".to_string());
    }

    None
}

fn send_notification(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary("Stop It - Pomodoro Alert")
        .body(message)
        .timeout(0) // No auto-dismiss
        .show()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    // Check for log file argument
    let log_file = if let Some(pos) = args.iter().position(|a| a == "--log" || a == "-l") {
        args.get(pos + 1).cloned()
    } else {
        Some(format!(
            "{}/.local/share/stop_it/activity.log", // TODO: check if making this dayly
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        ))
    };

    // Create log directory if needed
    if let Some(ref path) = log_file {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    println!("🍅 Stop It - Browser Activity Monitor & Pomodoro Timer");
    println!("======================================================");
    println!(
        "Pomodoro settings: {}min work / {}min break",
        pomodoro::pomodoro::POMODORO_WORK_MINUTES,
        pomodoro::pomodoro::POMODORO_BREAK_MINUTES
    );
    println!("Running on Hyprland (Wayland)");
    if verbose {
        println!("Verbose mode: ON");
    }
    if let Some(ref path) = log_file {
        println!("Logging to: {}", path);
    }
    println!("Monitoring active window... Press Ctrl+C to stop and see stats\n");

    let mut tracker = DomainTracker::new(log_file);
    let mut last_title = String::new();

    println!(
        "{} Starting in {} mode\n",
        tracker.mode.emoji(),
        tracker.mode.as_str()
    );

    loop {
        match get_active_window_title() {
            Ok(title) => {
                if !title.is_empty() {
                    if verbose && title != last_title {
                        println!("[DEBUG] Window title: {}", title);
                    }

                    let domain = extract_domain_from_title(&title);

                    if verbose && title != last_title {
                        println!("[DEBUG] Extracted domain: {:?}", domain);
                    }

                    if domain != tracker.current_domain {
                        if let Some(ref d) = domain {
                            let msg =
                                format!("[{}] Switched to: {}", Local::now().format("%H:%M:%S"), d);
                            println!("{}", msg);
                            tracker.log(&msg);
                        } else if tracker.current_domain.is_some() {
                            let msg = format!(
                                "[{}] Left browser (no domain detected)",
                                Local::now().format("%H:%M:%S")
                            );
                            println!("{}", msg);
                            tracker.log(&msg);
                        }
                    }

                    last_title = title;
                    tracker.update(domain);

                    if tracker.should_switch_mode() {
                        let message = match tracker.mode {
                            pomodoro::pomodoro::PomodoroMode::Work => format!(
                                "Work session complete! Time for a {}-minute break.",
                                pomodoro::pomodoro::POMODORO_BREAK_MINUTES
                            ),
                            pomodoro::pomodoro::PomodoroMode::Break => format!(
                                "Break is over! Starting {}-minute work session.",
                                pomodoro::pomodoro::POMODORO_WORK_MINUTES
                            ),
                        };

                        println!("\n🔔 {}", message);
                        tracker.log(&format!("🔔 {}", message));

                        if let Err(e) = send_notification(&message) {
                            eprintln!("Failed to send notification: {}", e);
                        }

                        if tracker.mode == pomodoro::pomodoro::PomodoroMode::Work {
                            tracker.print_stats();
                        }

                        tracker.switch_mode();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error getting window title: {}", e);
            }
        }

        thread::sleep(StdDuration::from_millis(POLL_INTERVAL_MS));
    }
}
