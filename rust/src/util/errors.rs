use std::fmt;

#[derive(Debug, Clone)]
pub enum ToksaveErrorKind {
    Tool,
    Install,
    Download,
    Network,
    HealthCheck,
    Integrity,
    Platform,
    Config,
    Io,
}

#[derive(Debug)]
pub struct ToksaveError {
    pub kind: ToksaveErrorKind,
    pub context: String,
    pub message: String,
    pub remediation: Option<String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ToksaveError {
    fn new(
        kind: ToksaveErrorKind,
        context: &str,
        message: &str,
        remediation: Option<&str>,
    ) -> Self {
        Self {
            kind,
            context: context.to_string(),
            message: message.to_string(),
            remediation: remediation.map(str::to_string),
            source: None,
        }
    }

    pub fn tool(context: &str, message: &str) -> Self {
        Self::new(ToksaveErrorKind::Tool, context, message, None)
    }
    pub fn install(context: &str, message: &str, remediation: Option<&str>) -> Self {
        Self::new(ToksaveErrorKind::Install, context, message, remediation)
    }
    pub fn download(context: &str, message: &str, url: &str, remediation: Option<&str>) -> Self {
        Self::new(
            ToksaveErrorKind::Download,
            context,
            &format!("{message} ({url})"),
            remediation,
        )
    }
    pub fn network(context: &str, message: &str, url: &str, remediation: Option<&str>) -> Self {
        Self::new(
            ToksaveErrorKind::Network,
            context,
            &format!("{message} ({url})"),
            remediation,
        )
    }
    pub fn integrity(context: &str, message: &str, remediation: Option<&str>) -> Self {
        Self::new(ToksaveErrorKind::Integrity, context, message, remediation)
    }
    pub fn platform(platform: &str, message: &str, remediation: Option<&str>) -> Self {
        Self::new(ToksaveErrorKind::Platform, platform, message, remediation)
    }
    pub fn config(path: &str, message: &str) -> Self {
        Self::new(ToksaveErrorKind::Config, path, message, None)
    }
}

impl fmt::Display for ToksaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.message)?;
        if let Some(rem) = &self.remediation {
            write!(f, " Remediation: {rem}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ToksaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|b| b as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for ToksaveError {
    fn from(err: std::io::Error) -> Self {
        Self::new(ToksaveErrorKind::Io, "io", &err.to_string(), None)
    }
}

pub type Result<T> = std::result::Result<T, ToksaveError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_context_and_message() {
        let e = ToksaveError::install("rtk", "failed", Some("run manually"));
        let s = e.to_string();
        assert!(s.contains("rtk"));
        assert!(s.contains("failed"));
        assert!(s.contains("run manually"));
    }

    #[test]
    fn io_error_converts() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "no file");
        let e: ToksaveError = io.into();
        assert!(matches!(e.kind, ToksaveErrorKind::Io));
    }

    #[test]
    fn config_error_has_no_remediation() {
        let e = ToksaveError::config("settings.json", "parse failed");
        assert!(!e.to_string().contains("Remediation"));
    }
}
