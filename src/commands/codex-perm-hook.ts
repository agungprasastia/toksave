import { readFileSync } from "node:fs";

const ALLOWLIST: Record<string, boolean> = {
  rtk: true,
  toksave: true,
  tokless: true,
  git: true,
  cd: true,
  ls: true,
  node: true,
  npm: true,
  npx: true,
  "context-mode": true,
  codegraph: true,
  cat: true,
  head: true,
  tail: true,
  grep: true,
  find: true,
  pwd: true,
  which: true,
  echo: true,
  true: true,
  false: true,
  bash: true,
};

function firstToken(s: string): string {
  const match = s.match(/^\s*(\S+)/);
  return match?.[1] ?? s;
}

function stripPath(tok: string): string {
  const lastSlash = Math.max(tok.lastIndexOf("/"), tok.lastIndexOf("\\"));
  if (lastSlash >= 0) {
    tok = tok.slice(lastSlash + 1);
  }
  if (process.platform === "win32") {
    tok = tok
      .replace(/\.exe$/i, "")
      .replace(/\.cmd$/i, "")
      .replace(/\.bat$/i, "");
  }
  return tok;
}

function bashInnerScriptAllAllowed(cmd: string): boolean {
  // Simplistic check for bash -c "..."
  const match = cmd.match(/bash\s+-c\s+["'](.*)["']/);
  if (match?.[1]) {
    const inner = match[1];
    const tok = stripPath(firstToken(inner));
    return !!ALLOWLIST[tok];
  }
  return false;
}

function isAllowed(toolName: string, command: string): boolean {
  if (toolName === "apply_patch") return true;
  if (toolName.startsWith("ctx_") || toolName.startsWith("codegraph_")) return true;
  if (toolName !== "Bash") return false;

  const cmd = command.trim();
  if (!cmd) return false;

  let tok = firstToken(cmd);
  tok = stripPath(tok);

  if (tok === "bash" || tok === "sh") {
    return bashInnerScriptAllAllowed(cmd);
  }

  return !!ALLOWLIST[tok];
}

export function runCodexPermHook(): number {
  try {
    const input = readFileSync(0, "utf-8"); // Read all from stdin
    if (!input) return 0;

    const req = JSON.parse(input) as { tool_name?: string; tool_input?: { command?: string } };
    const toolName = req.tool_name ?? "";
    const command = req.tool_input?.command ?? "";

    if (!isAllowed(toolName, command)) {
      return 0; // Do nothing, fallback to manual prompt
    }

    const resp = {
      hookSpecificOutput: {
        hookEventName: "PermissionRequest",
        decision: { behavior: "allow" },
      },
    };

    console.log(JSON.stringify(resp));
    return 0;
  } catch {
    return 0;
  }
}
