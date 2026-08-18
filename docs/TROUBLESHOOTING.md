# TokSave Troubleshooting & FAQ

Guide to resolving common issues, health check warnings, and environment-specific edge cases.

---

## Quick Diagnostic Checklist

If TokSave or your AI agents behave unexpectedly after wiring:

1. **Run Health Check**:
   ```bash
   toksave doctor
   ```
2. **Run Automatic Repair**:
   ```bash
   toksave doctor --fix
   ```
3. **Restart Your AI Agent**:
   Most AI agents (Claude Code, OpenCode, VS Code Copilot) only read updated JSON/TOML configurations on startup.

---

## Common Issues & Solutions

### 1. `os error 193` on Windows (`%1 is not a valid Win32 application`)
- **Cause**: Node.js `npm install -g` creates extensionless POSIX shell scripts alongside `.cmd` files in global bin directories. Spawning extensionless scripts directly on Windows causes execution errors.
- **Solution**: TokSave auto-resolves executables via Windows `PATHEXT` to pick `.cmd` twins. Ensure your system `PATH` includes the npm global directory (`%APPDATA%\npm` or `%LOCALAPPDATA%\Programs\toksave`).

---

### 2. Node.js Version Warning (`Node.js >= 22 required`)
- **Cause**: Tools like CodeGraph and Context-Mode require modern Node.js features (v22+).
- **Solution**: Upgrade Node.js via your package manager or official site:
  - **nvm**: `nvm install 22 && nvm use 22`
  - **fnm**: `fnm install 22 && fnm use 22`
  - **winget**: `winget install OpenJS.NodeJS.LTS`

---

### 3. Windows shells: cmd, PowerShell 5.1, PowerShell 7
- **Supported**: `cmd.exe`, Windows PowerShell 5.1 (`powershell.exe`), and PowerShell 7 (`pwsh`). Git Bash is not required.
- **Symptom**: a hooked command never runs, or PowerShell errors on `C:/Users/.../toksave.exe` / `C:/Users/.../rtk.exe` (it treats `C:` as Set-Location and `/Users/...` as a switch).
- **Cause**: older toksave versions wrote forward-slash hook paths for Git Bash. That form is valid in cmd and as MCP argv, but not as the first token of a PowerShell command.
- **Solution**: `toksave doctor --fix` (or re-run `toksave init`) rewrites hook `command` strings and RTK prefixes to backslash form. MCP `command` fields stay forward-slash on purpose (CreateProcess).

---

### 4. Caveman / GitHub Rate Limiting
- **Symptom**: `Caveman not installed` or `GitHub rate limit exceeded`.
- **Cause**: Caveman fetches `SKILL.md` from GitHub repositories. Unauthenticated GitHub API calls may hit rate limits (60 requests/hour).
- **Solution**: Wait for the rate limit window to reset or run `toksave init -t caveman` when network connectivity is clear. Local fallback skills are used automatically when available.

---

### 5. Manually Edited Agent Configs
- **Symptom**: Agent configuration fails to load after manual editing.
- **Solution**:
  1. Backup your custom config.
  2. Run `toksave uninstall` to clean TokSave blocks.
  3. Run `toksave init` to cleanly re-wire.

---

## Getting Help

If you encounter an issue not covered in this guide:
- Check existing [GitHub Issues](https://github.com/agungprasastia/toksave/issues).
- Run `toksave doctor -v` for verbose diagnostic output and attach it to your report.
