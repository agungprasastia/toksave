/** Context-Mode AGENTS.md routing rules — based on official mksglu/context-mode docs. */
export const CTX_RULES_BLOCK = `
<!-- CONTEXT-MODE_START — managed by toksave, do not edit -->
## context-mode — MANDATORY routing rules

context-mode MCP tools available. Rules protect context window from flooding. Follow strictly.

### Think in Code — MANDATORY

Analyze/count/filter/compare/search/parse/transform data: **write code** via ctx_execute, console.log() only the answer. Do NOT read raw data into context.

### BLOCKED — do NOT use

#### curl / wget — FORBIDDEN
Do NOT use curl/wget via run_command. Dumps raw HTTP into context.
Use: ctx_fetch_and_index(url, source) or ctx_execute with fetch()

#### Direct web fetching — FORBIDDEN
No read_url_content for large pages. Raw HTML can exceed 100 KB.
Use: ctx_fetch_and_index(url, source) then ctx_search(queries)

### REDIRECTED — use sandbox

#### Shell (>20 lines output)
run_command ONLY for: git, mkdir, rm, mv, cd, ls, npm install, pip install.
Otherwise: ctx_batch_execute or ctx_execute.

#### File reading (for analysis)
Reading to **edit** → view_file correct. Reading to **analyze/explore/summarize** → ctx_execute_file.

### Tool selection

1. **GATHER**: ctx_batch_execute — runs all commands, auto-indexes, returns search. ONE call.
2. **FOLLOW-UP**: ctx_search — all questions as array, ONE call.
3. **PROCESSING**: ctx_execute | ctx_execute_file — sandbox, only stdout enters context.
4. **WEB**: ctx_fetch_and_index then ctx_search — raw HTML never enters context.
5. **INDEX**: ctx_index — store in FTS5 for later search.

### Output

Write artifacts to FILES — never inline. Return: file path + 1-line description.
<!-- CONTEXT-MODE_END -->
`;

/** Check if context-mode rules are already present in a file. */
export function hasCtxRules(content: string): boolean {
  return content.includes("CONTEXT-MODE_START");
}

/** Remove context-mode rules from a file. */
export function removeCtxRules(content: string): string {
  return content
    .replace(/\n?<!-- CONTEXT-MODE_START[\s\S]*?CONTEXT-MODE_END -->\n?/g, "")
    .trim();
}
