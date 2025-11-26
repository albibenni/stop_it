use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HyprlandWindow {
    title: String,
}

pub fn get_active_window_title() -> Result<String, Box<dyn std::error::Error>> {
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
