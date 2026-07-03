/** RTK AGENTS.md/instructions.md rules — guides AI agents on RTK usage. */
export const RTK_RULES_BLOCK = `
<!-- RTK_START — managed by toksave, do not edit -->
## RTK — Token-Saving Command Wrapper

RTK wraps shell commands to compress output (60-90% token savings).

### Usage
For commands with large output, prefix with \`rtk\`:
- \`rtk npm run build\` — compress build logs
- \`rtk git log --all\` — compress git history
- \`rtk find . -name "*.ts"\` — compress file listings

### Automatic Prefixing
Some agents auto-prefix Bash commands via hooks. If RTK is already applied, you'll see compressed output automatically.

### When to Use Explicitly
Use \`rtk\` prefix for:
- Build/compile commands with verbose output
- Large file searches or listings
- Git operations returning many commits/files
- Package manager operations (npm, pip, cargo)
- Any command likely to produce >50 lines of output

Skip \`rtk\` for:
- Interactive commands (vim, nano, less)
- Commands needing raw output (jq, awk pipes)
- Single-line outputs (echo, pwd, which)
<!-- RTK_END -->
`;

/** Check if RTK rules are already present in a file. */
export function hasRtkRules(content: string): boolean {
  return /RTK_START\b/.test(content);
}

/** Remove RTK rules from a file. */
export function removeRtkRules(content: string): string {
  return content.replace(/\r?\n?<!--\s*RTK_START\b[\s\S]*?RTK_END\s*-->\r?\n?/g, "\n");
}
