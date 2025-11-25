use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

#[derive(Debug, Deserialize)]
pub struct TabUpdateMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub url: String,
    pub title: String,
    pub domain: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct NativeResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// Read a message from stdin using Chrome native messaging protocol
/// Messages are prefixed with a 4-byte length in native byte order
pub fn read_message() -> io::Result<Option<TabUpdateMessage>> {
    let mut length_bytes = [0u8; 4];

    // Read the 4-byte message length
    match io::stdin().read_exact(&mut length_bytes) {
        Ok(_) => {},
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            return Ok(None); // No more messages
        }
        Err(e) => return Err(e),
    }

    let length = u32::from_ne_bytes(length_bytes) as usize;

    // Read the message content
    let mut buffer = vec![0u8; length];
    io::stdin().read_exact(&mut buffer)?;

    // Parse JSON
    let message: TabUpdateMessage = serde_json::from_slice(&buffer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(Some(message))
}

/// Write a response to stdout using Chrome native messaging protocol
pub fn write_response(response: &NativeResponse) -> io::Result<()> {
    let json = serde_json::to_string(response)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let length = json.len() as u32;
    let length_bytes = length.to_ne_bytes();

    // Write length prefix
    io::stdout().write_all(&length_bytes)?;

    // Write message content
    io::stdout().write_all(json.as_bytes())?;
    io::stdout().flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_serialization() {
        let response = NativeResponse {
            success: true,
            message: Some("Test message".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"message\":\"Test message\""));
    }
}
