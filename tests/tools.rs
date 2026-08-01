mod common;

use common::setup;
use toksave::registry::{tool_installed_version, RunOpts, ToolId, ALL_TOOLS};
use toksave::tools::caveman::CavemanTool;
use toksave::tools::codegraph::CodegraphTool;
use toksave::tools::context_mode::ContextModeTool;
use toksave::tools::ponytail::PonytailTool;
use toksave::tools::principles::PrinciplesTool;
use toksave::tools::rtk::RtkTool;
use toksave::tools::{install_tool, tool_health_check, Tool};

#[test]
fn all_tools_registered_in_registry() {
    assert_eq!(ALL_TOOLS.len(), 6);
    let ids: Vec<_> = ALL_TOOLS.iter().map(|t| t.id).collect();
    assert!(ids.contains(&ToolId::Rtk));
    assert!(ids.contains(&ToolId::Caveman));
    assert!(ids.contains(&ToolId::Codegraph));
    assert!(ids.contains(&ToolId::ContextMode));
    assert!(ids.contains(&ToolId::Ponytail));
    assert!(ids.contains(&ToolId::Principles));
}

#[test]
fn health_check_and_version_for_all_tools() {
    let _env = setup();

    // Rtk
    let rtk_hc = tool_health_check(ToolId::Rtk);
    assert!(!rtk_hc.healthy || rtk_hc.version.is_some());
    assert_eq!(
        tool_installed_version(ToolId::Rtk),
        RtkTool.installed_version()
    );

    // Caveman
    let caveman_hc = tool_health_check(ToolId::Caveman);
    assert_eq!(
        caveman_hc.healthy,
        CavemanTool.installed_version().is_some()
    );
    assert_eq!(
        tool_installed_version(ToolId::Caveman),
        CavemanTool.installed_version()
    );

    // Codegraph
    let codegraph_hc = tool_health_check(ToolId::Codegraph);
    assert_eq!(
        codegraph_hc.healthy,
        CodegraphTool.installed_version().is_some()
    );
    assert_eq!(
        tool_installed_version(ToolId::Codegraph),
        CodegraphTool.installed_version()
    );

    // ContextMode
    let ctx_hc = tool_health_check(ToolId::ContextMode);
    assert_eq!(
        ctx_hc.healthy,
        ContextModeTool.installed_version().is_some()
    );
    assert_eq!(
        tool_installed_version(ToolId::ContextMode),
        ContextModeTool.installed_version()
    );

    // Ponytail
    let ponytail_hc = tool_health_check(ToolId::Ponytail);
    assert_eq!(
        ponytail_hc.healthy,
        PonytailTool.installed_version().is_some()
    );
    assert_eq!(
        tool_installed_version(ToolId::Ponytail),
        PonytailTool.installed_version()
    );

    // Principles
    let principles_hc = tool_health_check(ToolId::Principles);
    assert!(principles_hc.healthy);
    assert_eq!(
        tool_installed_version(ToolId::Principles),
        Some("instruction-only".to_string())
    );
    assert_eq!(
        PrinciplesTool.installed_version(),
        Some("instruction-only".to_string())
    );
}

#[tokio::test]
async fn install_dry_run_all_tools() {
    let _env = setup();
    let opts = RunOpts {
        dry_run: true,
        upgrade: false,
        verbose: false,
        yes: true,
        report: None,
    };

    for &tool in &[
        ToolId::Rtk,
        ToolId::Caveman,
        ToolId::Codegraph,
        ToolId::ContextMode,
        ToolId::Ponytail,
        ToolId::Principles,
    ] {
        let res = install_tool(tool, &opts).await;
        assert!(
            res.is_ok() && res.unwrap(),
            "Failed dry run install for {:?}",
            tool
        );
    }
}
