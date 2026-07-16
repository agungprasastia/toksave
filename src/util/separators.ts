import * as paths from "./paths.js";

const END_MARKER_RE = /<!-- [^>]+[_-]end(?:[^>]*)? -->/gi;
const START_MARKER_RE = /<!-- [^>]+[_-](?:start|begin)(?:[^>]*)? -->/gi;

function countNewlines(s: string): number {
  let n = 0;
  for (let i = 0; i < s.length; i++) if (s[i] === "\n") n++;
  return n;
}

export function normalizeSeparators(filePath: string): void {
  const raw = paths.readFile(filePath);
  if (!raw) return;

  // Find all end markers
  const endMarkers: { index: number; endIndex: number; text: string }[] = [];
  END_MARKER_RE.lastIndex = 0;
  for (;;) {
    const m = END_MARKER_RE.exec(raw);
    if (!m) break;
    endMarkers.push({ index: m.index, endIndex: m.index + m[0].length, text: m[0] });
  }

  if (endMarkers.length === 0) return;

  let result = raw;
  // Process from last to first to keep indices valid
  for (let i = endMarkers.length - 1; i >= 0; i--) {
    const em = endMarkers[i]!;
    const afterEnd = result.slice(em.endIndex);
    START_MARKER_RE.lastIndex = 0;
    const startMatch = START_MARKER_RE.exec(afterEnd);
    if (!startMatch) continue;
    const between = afterEnd.slice(0, startMatch.index);
    const nl = countNewlines(between);
    if (nl === 2) continue;
    const newBetween = "\n\n";
    result = result.slice(0, em.endIndex) + newBetween + result.slice(em.endIndex + between.length);
  }

  if (result !== raw) {
    paths.writeFile(filePath, result);
  }
}

function instructionPathForAgent(agent: string): string {
  switch (agent) {
    case "claude":
      return paths.claudePaths().agentsMd;
    case "opencode":
      return paths.opencodePaths().agentsMd;
    case "codex":
      return paths.codexPaths().instructions;
    case "antigravity":
      return paths.antigravityPaths().agentsMd;
    case "copilot":
      return paths.copilotPaths().instructions;
    case "droid":
      return paths.droidPaths().instructions;
    default:
      return "";
  }
}

export function ensureInstructionSeparators(agentIds: string[]): void {
  for (const id of agentIds) {
    const p = instructionPathForAgent(id);
    if (p) normalizeSeparators(p);
  }
}
