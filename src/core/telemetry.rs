use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub data: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

impl TelemetryEvent {
    pub fn new(event_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            event_type: event_type.into(),
            data,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_event_creation() {
        let event = TelemetryEvent::new(
            "test_event",
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(event.event_type, "test_event");
        assert!(!event.timestamp == 0);
    }
}

