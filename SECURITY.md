# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | ✅ Current release |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT** open a public issue.
2. Email **agungprasastia@gmail.com** with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
3. You will receive an acknowledgment within **48 hours**.
4. A fix will be released as soon as possible, and you will be credited (unless you prefer anonymity).

## Scope

toksave modifies configuration files for AI coding agents. Security concerns include:

- **Config injection** — malicious content written to agent config files.
- **Binary downloads** — RTK binaries downloaded from GitHub Releases.
- **Shell execution** — `rtk init -g` modifies shell profiles.
- **npm installs** — `npm install -g` for CodeGraph and Context-Mode.

## Mitigations

- All downloads use HTTPS.
- `--dry-run` flag allows previewing changes before committing.
- Manifest tracks what toksave modified for clean uninstall.
- No telemetry or data collection.
