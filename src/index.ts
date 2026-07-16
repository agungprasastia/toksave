#!/usr/bin/env bun
import { parseCli } from "./cli.js";
import * as doctor from "./commands/doctor.js";
import * as init from "./commands/init.js";
import * as selfUpdate from "./commands/self-update.js";
import * as uninstall from "./commands/uninstall.js";
import * as update from "./commands/update.js";

async function main(): Promise<void> {
  // Early hook dispatch to match tokless main.go behavior before commander parse
  const argv = process.argv.slice(2);
  if (argv.length >= 2) {
    const a0 = argv[0] ?? "";
    const a1 = argv[1] ?? "";
    // tokless compatibility: agy-hook codegraph-index / copilot-hook codegraph-index
    if (
      (a0 === "agy-hook" && a1 === "codegraph-index") ||
      (a0 === "copilot-hook" && a1 === "codegraph-index")
    ) {
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const { runCodegraphIndexHook } = require("./commands/codegraph-index-hook.js");
      const code = runCodegraphIndexHook();
      process.exit(code);
    }
    // rtk-hook variants: agy, codex, claude, copilot, droid
    if (a0 === "rtk-hook" && ["agy", "codex", "claude", "copilot", "droid"].includes(a1)) {
      const { runRtkHookVariant } = require("./commands/rtk-hook.js");
      const code = runRtkHookVariant(a1);
      process.exit(code);
    }
    // index --auto <agent> variant (tokless Droid: index --auto droid)
    if (a0 === "index" && a1 === "--auto") {
      const { runIndex } = require("./commands/build-index.js");
      const code = runIndex(true);
      process.exit(code);
    }
  }

  const cli = parseCli(process.argv);

  let code = 0;

  switch (cli.command) {
    case "init":
      code = await init.run(cli.agents, cli.tools, cli.opts);
      break;
    case "doctor":
      code = await doctor.run(cli.offline, cli.fix, cli.opts);
      break;
    case "update":
      code = await update.run(cli.opts);
      break;
    case "uninstall":
      code = await uninstall.run(cli.agents, cli.tools, cli.opts);
      break;
    case "disable": {
      const { run } = require("./commands/disable.js");
      code = await run(cli.agents, cli.tools, cli.opts);
      break;
    }
    case "self-update":
      code = await selfUpdate.run();
      break;
    case "codex-perm-hook": {
      const { runCodexPermHook } = require("./commands/codex-perm-hook.js");
      code = runCodexPermHook();
      break;
    }
    case "rtk-hook": {
      const { runRtkHook } = require("./commands/rtk-hook.js");
      code = runRtkHook();
      break;
    }
    case "context-mode-hook": {
      const { runContextModeHook } = require("./commands/context-mode-hook.js");
      code = runContextModeHook();
      break;
    }
    case "runmcp": {
      const { runMcp } = require("./commands/runmcp.js");
      code = await runMcp();
      break;
    }
    case "index": {
      const { runIndex } = require("./commands/build-index.js");
      const auto = (cli as unknown as Record<string, unknown>).auto as boolean | undefined;
      code = runIndex(auto ?? false);
      break;
    }
    case "agy-hook":
    case "copilot-hook": {
      const { runCodegraphIndexHook } = require("./commands/codegraph-index-hook.js");
      code = runCodegraphIndexHook();
      break;
    }
  }

  process.exit(code);
}

main().catch((err) => {
  console.error(`\n  Error: ${err.message ?? err}\n`);
  process.exit(1);
});
