use toksave::content;

#[test]
fn test_managed_content_constants_not_empty() {
    assert!(!content::agent_instructions().is_empty());
    assert!(!content::caveman_skill().is_empty());
    assert!(!content::ctx_rules().is_empty());
}
