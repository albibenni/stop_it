use chrono::{DateTime, Local};
use notify_rust::Notification;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration as StdDuration;

const POLL_INTERVAL_MS: u64 = 1000; // Check active window every second
const POMODORO_WORK_MINUTES: i64 = 25; // Default Pomodoro work time
const POMODORO_BREAK_MINUTES: i64 = 5; // Default Pomodoro break time

#[derive(Debug, Deserialize)]
struct HyprlandWindow {
    title: String,
}

#[derive(Debug)]
struct DomainTracker {
    time_spent: HashMap<String, i64>, // domain -> seconds
    current_domain: Option<String>,
    session_start: DateTime<Local>,
    last_notification: Option<DateTime<Local>>,
}

impl DomainTracker {
    fn new() -> Self {
        Self {
            time_spent: HashMap::new(),
            current_domain: None,
            session_start: Local::now(),
            last_notification: None,
        }
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

    fn should_notify_break(&mut self) -> bool {
        let session_minutes = self.get_session_duration() / 60;

        if session_minutes >= POMODORO_WORK_MINUTES {
            if let Some(last) = self.last_notification {
                let minutes_since_last = (Local::now() - last).num_minutes();
                if minutes_since_last >= (POMODORO_WORK_MINUTES + POMODORO_BREAK_MINUTES) {
                    self.last_notification = Some(Local::now());
                    return true;
                }
            } else {
                self.last_notification = Some(Local::now());
                return true;
            }
        }
        false
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
    Ok(window.title)
}

fn extract_domain_from_title(title: &str) -> Option<String> {
    let domain_patterns = vec![
        Regex::new(r"https?://(?:www\.)?([a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+)").unwrap(),
        Regex::new(r"(?:^|\s|\()([a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+\.[a-zA-Z]{2,})").unwrap(),
        Regex::new(r"(?:^|\s|\()([a-zA-Z0-9-]+\.(com|org|net|io|dev|co))").unwrap(),
    ];

    for pattern in domain_patterns {
        if let Some(captures) = pattern.captures(title) {
            if let Some(domain) = captures.get(1) {
                return Some(domain.as_str().to_lowercase());
            }
        }
    }

    if title.contains("YouTube") {
        return Some("youtube.com".to_string());
    } else if title.contains("Reddit") {
        return Some("reddit.com".to_string());
    } else if title.contains("Twitter") || title.contains("X.com") {
        return Some("twitter.com".to_string());
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

    println!("🍅 Stop It - Browser Activity Monitor & Pomodoro Timer");
    println!("======================================================");
    println!(
        "Pomodoro settings: {}min work / {}min break",
        POMODORO_WORK_MINUTES, POMODORO_BREAK_MINUTES
    );
    println!("Running on Hyprland (Wayland)");
    if verbose {
        println!("Verbose mode: ON");
    }
    println!("Monitoring active window... Press Ctrl+C to stop and see stats\n");

    let mut tracker = DomainTracker::new();
    let mut last_title = String::new();

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
                            println!("[{}] Switched to: {}", Local::now().format("%H:%M:%S"), d);
                        } else if tracker.current_domain.is_some() {
                            println!("[{}] Left browser (no domain detected)", Local::now().format("%H:%M:%S"));
                        }
                    }

                    last_title = title;
                    tracker.update(domain);

                    if tracker.should_notify_break() {
                        let message = format!(
                            "You've been working for {} minutes! Time for a {}-minute break.",
                            POMODORO_WORK_MINUTES, POMODORO_BREAK_MINUTES
                        );
                        println!("\n🔔 {}", message);
                        if let Err(e) = send_notification(&message) {
                            eprintln!("Failed to send notification: {}", e);
                        }
                        tracker.print_stats();
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
