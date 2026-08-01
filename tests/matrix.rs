use toksave::registry::{unwire_tool, verify_tool, wire_tool, ALL_AGENTS, ALL_TOOLS};

mod common;

#[tokio::test]
async fn test_full_8x6_agent_tool_matrix() {
    let _env = common::setup();
    let opts = toksave::registry::RunOpts::default();

    for agent in ALL_AGENTS {
        for tool in ALL_TOOLS {
            let wire_res = wire_tool(agent.id, tool.id, &opts).await;
            assert!(
                wire_res.is_ok(),
                "wire_tool failed for agent {:?}, tool {:?}",
                agent.id,
                tool.id
            );

            let is_wired = verify_tool(agent.id, tool.id);
            assert_eq!(
                is_wired,
                Some(true),
                "verify_tool returned not wired for agent {:?}, tool {:?}",
                agent.id,
                tool.id
            );

            let unwire_res = unwire_tool(agent.id, tool.id, &opts).await;
            assert!(
                unwire_res.is_ok(),
                "unwire_tool failed for agent {:?}, tool {:?}",
                agent.id,
                tool.id
            );

            let is_wired_after = verify_tool(agent.id, tool.id);
            assert_eq!(
                is_wired_after,
                Some(false),
                "verify_tool returned wired after unwire for agent {:?}, tool {:?}",
                agent.id,
                tool.id
            );
        }
    }
}
