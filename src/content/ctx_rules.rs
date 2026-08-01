pub const CTX_RULES_MD: &str = r#"
Use ctx tools before raw file reads for large blobs (200+ lines) or multi-file operations.
"#;

pub fn ctx_rules() -> &'static str {
    CTX_RULES_MD
}
