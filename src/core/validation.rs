use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

/// Pipeline validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Validate pipeline definition
pub fn validate_pipeline(pipeline: &Value) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check required fields
    if !pipeline.is_object() {
        result.add_error("Pipeline must be a JSON object".to_string());
        return result;
    }

    let obj = pipeline.as_object().unwrap();

    // Required: name
    if !obj.contains_key("name") {
        result.add_error("Pipeline must have a 'name' field".to_string());
    } else if let Some(name) = obj.get("name") {
        if !name.is_string() || name.as_str().unwrap().is_empty() {
            result.add_error("Pipeline 'name' must be a non-empty string".to_string());
        }
    }

    // Required: version
    if !obj.contains_key("version") {
        result.add_error("Pipeline must have a 'version' field".to_string());
    } else if let Some(version) = obj.get("version") {
        if !version.is_string() {
            result.add_error("Pipeline 'version' must be a string".to_string());
        }
    }

    // Validate steps if present
    if let Some(steps) = obj.get("steps") {
        if !steps.is_array() {
            result.add_error("Pipeline 'steps' must be an array".to_string());
        } else {
            let steps_array = steps.as_array().unwrap();
            if steps_array.is_empty() {
                result.add_warning("Pipeline has no steps defined".to_string());
            }

            for (idx, step) in steps_array.iter().enumerate() {
                if !step.is_object() {
                    result.add_error(format!("Step {} must be an object", idx));
                    continue;
                }

                let step_obj = step.as_object().unwrap();
                if !step_obj.contains_key("type") {
                    result.add_error(format!("Step {} must have a 'type' field", idx));
                }
            }
        }
    }

    // Validate metadata if present
    if let Some(metadata) = obj.get("metadata") {
        if !metadata.is_object() {
            result.add_warning("Pipeline 'metadata' should be an object".to_string());
        }
    }

    debug!(
        "Pipeline validation: {} errors, {} warnings",
        result.errors.len(),
        result.warnings.len()
    );

    result
}

/// Validate telemetry event
pub fn validate_event(event: &Value) -> ValidationResult {
    let mut result = ValidationResult::new();

    if !event.is_object() {
        result.add_error("Event must be a JSON object".to_string());
        return result;
    }

    let obj = event.as_object().unwrap();

    // Required: timestamp
    if !obj.contains_key("timestamp") {
        result.add_error("Event must have a 'timestamp' field".to_string());
    }

    // Required: source
    if !obj.contains_key("source") {
        result.add_error("Event must have a 'source' field".to_string());
    }

    // Required: data
    if !obj.contains_key("data") {
        result.add_error("Event must have a 'data' field".to_string());
    }

    result
}

/// Validate batch of events
pub fn validate_batch(events: &[Value]) -> ValidationResult {
    let mut result = ValidationResult::new();

    if events.is_empty() {
        result.add_error("Batch cannot be empty".to_string());
        return result;
    }

    if events.len() > 1000 {
        result.add_warning(format!(
            "Large batch size: {} events (recommended: < 1000)",
            events.len()
        ));
    }

    for (idx, event) in events.iter().enumerate() {
        let event_result = validate_event(event);
        if !event_result.is_valid {
            for error in event_result.errors {
                result.add_error(format!("Event {}: {}", idx, error));
            }
        }
        result.warnings.extend(event_result.warnings);
    }

    result
}

