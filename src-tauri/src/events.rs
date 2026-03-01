use serde::Serialize;

/// Valid event types that map to Claude Code hook events
pub(crate) const VALID_EVENTS: &[&str] =
    &["complete", "error", "attention", "stop", "session-end"];

/// Payload emitted to the webview
#[derive(Debug, Clone, Serialize)]
pub struct ClippyEvent {
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Check if an event type string is valid
pub fn is_valid_event(event_type: &str) -> bool {
    VALID_EVENTS.contains(&event_type)
}

/// Extract event type from URL path segment, returns None if invalid
#[allow(dead_code)]
pub fn event_type_from_path(path: &str) -> Option<&str> {
    let path = path.trim_matches('/');
    if is_valid_event(path) {
        Some(path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_from_path() {
        assert_eq!(event_type_from_path("complete"), Some("complete"));
        assert_eq!(event_type_from_path("error"), Some("error"));
        assert_eq!(event_type_from_path("attention"), Some("attention"));
        assert_eq!(event_type_from_path("stop"), Some("stop"));
        assert_eq!(event_type_from_path("session-end"), Some("session-end"));
        assert_eq!(event_type_from_path("unknown"), None);
        assert_eq!(event_type_from_path(""), None);
    }

    #[test]
    fn test_valid_event_types() {
        let valid = ["complete", "error", "attention", "stop", "session-end"];
        for event_type in &valid {
            assert!(
                is_valid_event(event_type),
                "{} should be valid",
                event_type
            );
        }
    }

    #[test]
    fn test_invalid_event_types() {
        let invalid = ["", "unknown", "hack", "../etc/passwd", "complete/extra"];
        for event_type in &invalid {
            assert!(
                !is_valid_event(event_type),
                "{} should be invalid",
                event_type
            );
        }
    }
}
