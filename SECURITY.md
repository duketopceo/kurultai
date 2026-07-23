# Security Policy

## Supported Versions

Currently this is an active development project (v0.1.0). Security patches will be applied to the main branch and included in future releases.

## Reporting a Vulnerability

If you discover a security vulnerability in Kurultai, please report it responsibly.

### How to Report
**Do not** open a public issue for security vulnerabilities.

Instead, send an email to: [duketopceo@gmail.com](mailto:duketopceo@gmail.com)

Include the following information in your report:
- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact of the vulnerability
- Any suggested mitigation (if known)

### What to Expect
- You will receive an acknowledgment within 48 hours
- We will assess the severity and develop a fix
- We will coordinate disclosure timing with you
- You will be credited in the security advisory (unless you prefer anonymity)

### Security Best Practices for Contributors
- Never commit API keys, tokens, or credentials
- Use environment variables for sensitive configuration
- Review dependencies for known vulnerabilities (`cargo audit`)
- Follow secure coding practices for Rust
- Be cautious with user input handling and validation

### Known Security Considerations
- The project stores knowledge atoms in SQLite - ensure proper file permissions
- MCP connectors may expose local file systems - configure appropriately
- API keys for OpenRouter are stored in user config - protect config files
- The daemon (when implemented) will need proper authentication for network access

## Security Updates
Security fixes will be:
- Applied to the main branch immediately
- Included in the next release
- Documented in release notes

We aim to patch critical vulnerabilities within 7 days of responsible disclosure.