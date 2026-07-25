---
tags: [agent-zero, installer, dx, v1]
related:
  - docs/agent-zero/INDEX.md
  - docs/plans/2026-07-25-005-feat-v1-personal-installer-plan.md
---
# ISSUE: Agent-Level CLI Installer

**Labels:** feature, developer-experience, agent-integration
**Tracking:** #6 (Phase 2) | Related: #5 (MCP Tools)
**Status:** Open
**Created:** 2026-07-25

---

## Problem

Currently, kurultai has no automated installer. Users must:

1. **Manually install Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cmd /c rustup-init.bat  # On Windows
```

2. **Clone and build manually**
```bash
git clone https://github.com/duketopceo/kurultai.git
cd kurultai
cargo build --release
```

3. **Configure manually**
```bash
cp config.toml.example ~/.config/kurultai/config.toml
nano ~/.config/kurultai/config.toml  # Edit sources, API keys, etc.
```

4. **Index manually**
```bash
kurultai init --agent cursor
kurultai index --full
```

**Impact:** High barrier to entry, poor developer experience, manual errors common.

---

## Proposed Solution

### Agent-Level Install Command

**Personal Install:**
```bash
hey claude install kurulatai brain from github

→ Kurultai Installer 1.0

⚙️ Detecting environment... ✓ (Linux, Zsh)

📦 Installing Rust... ✓ (cargo 1.97.1)

📥 Downloading kurultai... ✓ (607 objects, 360KB)

🔨 Building kurultai... ✓ (dev mode, 57s)

🔧 Creating config... ✓ (~/.config/kurultai/config.toml)

📝 Setting up markdown source... ✓ (./my-vault/

✅ Kurultai brain installed!

Quick start:
  kurultai index --full
  kurultai search "my todo list"

Need help? kurultai --help
```

**Team Install:**
```bash
hey claude "install kurulatai brain for my team, centralize under /data/kurultai/"

→ Kurultai Team Installer

🔍 Detecting team setup... ✓ (3 developers, 5 markdown vaults)

📋 Proposed configuration:
  - Central storage: /data/kurultai/cluster.db
  - Shared sources: 5 markdown vaults, 2 GitHub repos
  - Scheduling: 30min poll interval (low-latency)

⚠️ This will create:
  - Shared daemon on port 8421 (HTTP)
  - Config: /etc/kurultai/config.toml (team-level)
  - MCP wiring: per-developer ~/.config/kurultai/

Proceed? [Y/n] Y

✅ Team Kurultai brain installed!

Team dashboard: http://localhost:8421
```

**Company Install:**
```bash
hey claude "install kurulatai brain company-wide with RBAC"

→ Kurultai Company Installer

🏢 Enterprise Configuration:
  - Cluster: 3 nodes (us-east, us-west, eu-central)
  - Shared database: PostgreSQL with read replicas
  - Auth: SSO integration (Okta/Auth0)
  - RBAC: teams/groups/roles (admin, viewer, contributor)
  - Audit logging: all queries logged to PostgreSQL

📋 Components:
  - Kurultai daemon (HTTP + MCP)
  - Dashboard (Perplexity Brain UI)
  - API gateway (auth + rate limiting)
  - Scheduler service (overnight context update)
  - Export service (Obsidian/Notion sync)

💰 Estimated cost: ~$50/mo (OpenRouter API keys for embeddings)

Proceed? [Y/n] Y

✅ Company Kurultai brain installed!

Documentation: docs/company.md
Admin portal: https://kurultai.yourcompany.com
```

### Installer Architecture

```
┌─────────────────────────────────────────────────┐
│ Agent-Level Installer (Phase 2)                  │
├─────────────────────────────────────────────────┤
│                                                 │
│ 1. Environment Detection                         │
│    - OS: Linux/macOS/Windows                    │
│    - Shell: bash/zsh/fish/powershell            │
│    - Rust: install if missing (check binary)   │
│    - Git: install if missing                    │
│                                                 │
│ 2. Project Download                              │
│    - Clone kurultai repo                       │
│    - Pull latest version                        │
│    - Verify checksum (optional security check)  │
│                                                 │
│ 3. Build Process                                 │
│    - cargo build --release                      │
│    - Validate binary executable                 │
│    - Test basic CLI: kurultai --help           │
│                                                 │
│ 4. Configuration Generation                     │
│    - ~/.config/kurultai/config.toml            │
│    - Prompt for: sources, API keys, config      │
│    - Copy templates: .example → config.toml    │
│                                                 │
│ 5. Source Setup                                 │
│    - Detect markdown vaults                    │
│    - Detect GitHub repos                        │
│    - Detect AppFlowy workspace                  │
│    - Ask user to specify sources                │
│                                                 │
│ 6. MCP Wiring                                     │
│    - Detect MCP agents: Cursor, Claude Code, etc.│
│    - Create kurultai integration file            │
│    - Test MCP tools registration                 │
│                                                 │
│ 7. Indexing (Optional)                          │
│    - Initialize kurultai store                  │
│    - Run kurultai index --full                  │
│    - Show initial atom count                    │
│                                                 │
│ 8. Post-Install Help                             │
│    - Show quick start guide                     │
│    - kurultai --help                           │
│    - Show dashboard URL (if applicable)         │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Project Structure

```
/a0/usr/workdir/documents/github/kurultai/
├── .github/PR_DRAFTS/
│   └── ISSUE-004-agent-installer.md        ← Installer spec
├── scripts/
│   └── install/
│       ├── install.sh                        ← Linux/macOS installer
│       ├── install.ps1                       ← Windows installer
│       └── install.bat                       ← Windows legacy
├── docs/
│   └── agent-integration/
│       ├── personal-install.md               ← Personal setup guide
│       ├── team-install.md                   ← Team setup guide
│       └── company-install.md                ← Company setup guide
├── bin/
│   └── kurultai-installer                    ← Go/Rust CLI tool
└── tests/
    └── test-installer.sh                    ← Installer test suite
```

### Team Installer Details

**Multi-developer setup:**
```bash
# team-install.sh
#!/bin/bash

INSTALL_DIR="/data/kurultai"
CONFIG_DIR="/etc/kurultai"
USER_CONFIG_DIR="$HOME/.config/kurultai"

# Create shared storage
c mkdir -p "$INSTALL_DIR"
c mkdir -p "$CONFIG_DIR"

# Generate cluster-level config
cat > "$CONFIG_DIR/config.toml" << EOF
[cluster]
enabled = true
port = 8421
cluster_path = "$INSTALL_DIR/cluster.db"
EOF

# Create per-developer configs
c for dev in alice bob charlie; do
  mkdir -p "$USER_CONFIG_DIR/$dev"
  cp "$CONFIG_DIR/config.toml" "$USER_CONFIG_DIR/$dev/"
  echo "configured for $dev"
done

# Start shared daemon
curultai daemon --port 8421 --config "$CONFIG_DIR/config.toml" &
```

### Company Installer Details

**Enterprise setup:**
```bash
# company-install.sh
#!/bin/bash

COMPANY_DOMAIN="yourcompany.com"
ADMIN_EMAIL="admin@$COMPANY_DOMAIN"
SSO_ISSUER="https://sso.yourcompany.com"

# PostgreSQL database
# - 3 replicas: us-east, us-west, eu-central
# - Connection string: postgresql://kurultai:***@cluster.yourcompany.com:5432/kurultai

# RBAC configuration
# - roles: admin, viewer, contributor, auditor
# - permissions: read, write, delete, configure

# SSO integration
# - Okta/Auth0/Google Workspace
# - Users sync: AD/LDAP → Kurultai users

# Components to install
COMPONENTS=(
  "kurultai-daemon:port 8421"
  "kurultai-dashboard:port 8422"
  "kurultai-api-gateway:port 443"
  "kurultai-scheduler:hourly cron"
  "kurultai-export:obsidian/notion"
)

# Generate Helm chart (Kubernetes)
helm upgrade --install kurultai kurultai/helm/
```

### CLI Command Interface

```rust
// bin/kurultai-installer
#[tokio::main]
async fn main() {
    let matches = App::new("kurultai-installer")
        .subcommand(
            SubCommand::with_name("install")
            .arg(Arg::new("mode").short('m').takes_value(true).possible_values(&["personal", "team", "company"]).default_value("personal"))
            .arg(Arg::new("verbose").short('v').long("verbose"))
            .arg(Arg::new("config").short('c').takes_value(true).multiple(true))
            .arg(Arg::new("skip-build").long("skip-build"))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("install", m)) => {
            let mode = m.value_of("mode").unwrap_or("personal");
            let verbose = m.is_present("verbose");
            let skip_build = m.is_present("skip-build");
            
            match mode {
                "personal" => installer::personal(verbose, skip_build).await,
                "team" => installer::team(verbose).await,
                "company" => installer::company(verbose).await,
                _ => Err("Invalid mode"),
            }
        }
        _ => help(),
    }
}
```

### Success Metrics

- **Install time:** <2 minutes for personal install
- **Error rate:** <5% error rate during automated installation
- **Team adoption:** 50% of team members use automated install (vs manual)
- **Company setup:** 90% automation for company installer (no manual steps)
- **User satisfaction:** 80% report "easiest tool I've installed"

---

## Implementation Plan

### Phase 2: Personal Installer (Week 1-2)
- [ ] Add `bin/kurultai-installer` binary (Rust)
- [ ] Implement environment detection (OS, shell, Rust, Git)
- [ ] Implement project download (git clone)
- [ ] Implement build automation (cargo build)
- [ ] Implement config template generation
- [ ] Test on Linux and macOS

### Phase 3: Team Installer (Week 3-4)
- [ ] Add `team` install mode
- [ ] Implement multi-user config generation
- [ ] Implement shared daemon startup
- [ ] Test on shared Linux server

### Phase 4: Company Installer (Month 2)
- [ ] Add `company` install mode
- [ ] Implement PostgreSQL schema migration
- [ ] Implement SSO integration (Okta/Auth0)
- [ ] Implement RBAC configuration
- [ ] Implement Kubernetes Helm chart

---

## Testing

**Manual Tests:**
```bash
# 1. Test personal install on Linux
currupt kurultai-installer install --mode personal --verbose

# 2. Verify install location
ls ~/.config/kurultai/

# 3. Verify kurultai binary
~/.cargo/bin/kurultai --help

# 4. Test team install
./install.sh team --dev-alice alice@company.com

# 5. Verify per-user configs
ls ~/.config/kurultai/alice/
ls ~/.config/kurultai/bob/
```

**Integration Tests:**
- [ ] Test environment detection on Linux, macOS, Windows
- [ ] Test Rust installation (skip if present, install if missing)
- [ ] Test git clone and verify checksum
- [ ] Test kurultai build (--release flag)
- [ ] Test config file generation from templates
- [ ] Test MCP wiring (Cursor, Claude Code)
- [ ] Test team multi-user setup
- [ ] Test company SSO integration

---

## Acceptance Criteria

1. ✅ `kurultai-installer install` works on Linux, macOS, Windows
2. ✅ Installer detects and installs Rust if missing
3. ✅ Installer clones kurultai repo and builds successfully
4. ✅ Installer generates `~/.config/kurultai/config.toml`
5. ✅ Installer prompts for sources and API keys (optional)
6. ✅ Installer can be re-run idempotently (skip if installed)
7. ✅ Team installer creates per-developer configs
8. ✅ Team installer starts shared daemon on port 8421
9. ✅ Company installer creates PostgreSQL schema and SSO config
10. ✅ Company installer deploys Kubernetes Helm chart

---

## Open Questions

1. **Should installer create a system-wide binary symlink?**
   - Option A: Yes, `/usr/local/bin/kurultai` (easier to use)
   - Option B: No, require full path (isolated install)
   - Decision: Yes, create symlink with prompt confirmation

2. **Should installer skip build on dev machines?**
   - Option A: Yes, just clone and show build instructions
   - Option B: No, always build for reproducibility
   - Decision: Always build for reproducibility (CI testing)

3. **Should installer support internationalization?**
   - Option A: Yes, i18n strings for non-English users
   - Option B: No, English only (simplest)
   - Decision: English only initially (i18n in Phase 4+)

4. **Should installer delete git clone after install?**
   - Option A: Yes, free disk space
   - Option B: No, keep for future updates
   - Decision: Keep clone, add cleanup flag (`--clean`)

---

## References

- Master plan: [#27 — Work Order: Master phase plan](https://github.com/duketopceo/kurultai/issues/27)
- Phase 2 tracking: [#6 Search & Retrieval](https://github.com/duketopceo/kurultai/issues/6)
- MCP tools: [#5 MCP Slice](https://github.com/duketopceo/kurultai/issues/5)
- Perplexity Brain setup: https://www.perplexity.ai/help-center/en/articles/19700001-what-is-brain
- Tech: `tokio::process::Command` for OS commands, `migrate-repository` for database migrations