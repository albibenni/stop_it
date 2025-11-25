pub const POLL_INTERVAL_MS: u64 = 1000; // Check active window every second
pub const POMODORO_WORK_MINUTES: i64 = 25; // Default Pomodoro work time
pub const POMODORO_BREAK_MINUTES: i64 = 5; // Default Pomodoro break time

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PomodoroMode {
    Work,
    Break,
}

impl PomodoroMode {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            PomodoroMode::Work => "WORK",
            PomodoroMode::Break => "BREAK",
        }
    }

    pub(crate) fn emoji(&self) -> &str {
        match self {
            PomodoroMode::Work => "💼",
            PomodoroMode::Break => "☕",
        }
    }
}
