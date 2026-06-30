import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { installedVersion as codegraphInstalled, indexProject } from "../tools/codegraph.js";

const PROJECT_MARKERS = [
  ".git",
  "package.json",
  "go.mod",
  "Cargo.toml",
  "pyproject.toml",
  "pom.xml",
  "build.gradle",
  "tsconfig.json",
  "requirements.txt",
];

function looksLikeProject(dir: string): boolean {
  for (const m of PROJECT_MARKERS) {
    if (existsSync(join(dir, m))) {
      return true;
    }
  }
  return false;
}

function findProjectDir(dir: string): string {
  let current = dir;
  while (true) {
    if (looksLikeProject(current)) {
      return current;
    }
    const parent = dirname(current);
    if (parent === current || parent === ".") {
      break;
    }
    current = parent;
  }
  return dir;
}

export function runIndex(auto = false): number {
  let dir = process.cwd();

  if (auto) {
    dir = findProjectDir(dir);
    if (!looksLikeProject(dir)) {
      return 0; // Silently skip if auto and no project found
    }
  }

  if (!auto) {
    console.log("");
    console.log(
      `  \x1b[1;36mtoksave index\x1b[0m  \x1b[90mbuild per-project indexes in ${dir}\x1b[0m`,
    );
    console.log("");
  }

  const installed = codegraphInstalled();
  if (!installed) {
    if (!auto) {
      console.log(`  \x1b[90m• \x1b[0mCodeGraph  \x1b[90mnot installed — run toksave first\x1b[0m`);
    }
    return 1;
  }

  const ok = indexProject(dir);
  if (!auto) {
    if (ok) {
      console.log(`  \x1b[32m✔ \x1b[0mCodeGraph  \x1b[90mindexed\x1b[0m`);
    } else {
      console.log(`  \x1b[31m✖ \x1b[0mCodeGraph  \x1b[90mfailed to index\x1b[0m`);
    }
  }

  return ok ? 0 : 1;
}
