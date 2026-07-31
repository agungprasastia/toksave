#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub severity: Severity,
    pub message: String,
    pub remediation: Option<String>,
}

impl HealthIssue {
    pub fn error(message: &str, remediation: &str) -> Self {
        Self {
            severity: Severity::Error,
            message: message.to_string(),
            remediation: Some(remediation.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub version: Option<String>,
    pub issues: Vec<HealthIssue>,
}
