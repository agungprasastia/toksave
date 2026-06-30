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
      code = await doctor.run(cli.offline);
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
  }

  process.exit(code);
}

main().catch((err) => {
  console.error(`\n  Error: ${err.message ?? err}\n`);
  process.exit(1);
});
