use chrono::{DateTime, Local};
use notify_rust::Notification;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, interval};
mod pomodoro;
mod ws;

#[derive(Debug)]
struct DomainTracker {
    time_spent: HashMap<String, i64>, // domain -> seconds
    current_domain: Option<String>,
    session_start: DateTime<Local>,
    mode: pomodoro::pomodoro::PomodoroMode,
    mode_start: DateTime<Local>,
}

impl DomainTracker {
    fn new() -> Self {
        let now = Local::now();
        println!(
            "=== Session started at {} ===",
            now.format("%Y-%m-%d %H:%M:%S")
        );
        Self {
            time_spent: HashMap::new(),
            current_domain: None,
            session_start: now,
            mode: pomodoro::pomodoro::PomodoroMode::Work,
            mode_start: now,
        }
    }

    fn get_mode_duration(&self) -> i64 {
        (Local::now() - self.mode_start).num_seconds()
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

fn send_notification(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary("Stop It - Pomodoro Alert")
        .body(message)
        .timeout(0) // No auto-dismiss
        .show()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_daemon_mode().await
}

/// Run in daemon mode - WebSocket server + Pomodoro timer + activity tracking
async fn run_daemon_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍅 Stop It - Daemon Mode");
    println!("======================================================");
    println!(
        "Pomodoro settings: {}min work / {}min break",
        pomodoro::pomodoro::POMODORO_WORK_MINUTES,
        pomodoro::pomodoro::POMODORO_BREAK_MINUTES
    );
    println!("Running WebSocket server on ws://127.0.0.1:8765");
    println!("Tracking browser activity via WebSocket\n");

    // Create activity channel for browser messages
    let (activity_tx, mut activity_rx) = ws::websocket_server::create_activity_channel();

    // Shared tracker wrapped in Arc<Mutex<>> for thread-safe access
    let tracker = Arc::new(Mutex::new(DomainTracker::new()));
    let tracker_clone = Arc::clone(&tracker);

    // Spawn WebSocket server
    let ws_addr = "127.0.0.1:8765".parse()?;
    tokio::spawn(async move {
        if let Err(e) = ws::websocket_server::start_websocket_server(ws_addr, activity_tx).await {
            eprintln!("WebSocket server error: {}", e);
        }
    });

    // Spawn browser activity processor
    tokio::spawn(async move {
        while let Some(message) = activity_rx.recv().await {
            if let Ok(mut tracker) = tracker_clone.lock() {
                let domain = message.domain.clone().or_else(|| {
                    // Fallback: try to extract domain from URL
                    message.url.split('/').nth(2).map(|s| s.to_string())
                });

                if domain != tracker.current_domain {
                    if let Some(ref d) = domain {
                        let msg = format!(
                            "[{}] Browser switched to: {}",
                            Local::now().format("%H:%M:%S"),
                            d
                        );
                        println!("{}", msg);
                    }
                }

                tracker.update(domain);
            }
        }
    });

    // Main loop: Pomodoro timer
    let mut timer_interval = interval(Duration::from_secs(1));

    loop {
        timer_interval.tick().await;

        if let Ok(mut tracker) = tracker.lock() {
            // Update time for current domain
            if let Some(current) = tracker.current_domain.clone() {
                *tracker.time_spent.entry(current).or_insert(0) += 1;
            }

            // Check if should switch Pomodoro mode
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
}
