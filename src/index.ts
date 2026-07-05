#!/usr/bin/env bun
import { parseCli } from "./cli.js";
import * as doctor from "./commands/doctor.js";
import * as init from "./commands/init.js";
import * as selfUpdate from "./commands/self-update.js";
import * as uninstall from "./commands/uninstall.js";
import * as update from "./commands/update.js";

async function main(): Promise<void> {
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
      code = runIndex();
      break;
    }
  }

  process.exit(code);
}

main().catch((err) => {
  console.error(`\n  Error: ${err.message ?? err}\n`);
  process.exit(1);
});
