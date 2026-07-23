# Contributing to Kurultai

Thank you for your interest in contributing to Kurultai! This document provides guidelines for contributing to the project.

## Development Setup

### Prerequisites
- Rust 1.70+ (see `rust-toolchain.toml`)
- Git
- For MCP integration: Cursor or Claude Code

### Setup
```bash
# Clone the repository
git clone https://github.com/duketopceo/kurultai.git
cd kurultai

# Build in development mode
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=kurultai=debug cargo run -- status
```

### Development Workflow
```bash
# Format code
cargo fmt

# Lint
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test --locked

# Security audit
cargo audit
```

## Coding Standards

### Rust Conventions
- Use `cargo fmt` for formatting
- Pass `cargo clippy` with `-D warnings`
- Write tests for new functionality
- Use `anyhow::Result` for error handling
- Follow existing module structure in `src/`

### Documentation
- Add doc comments to public APIs
- Update README.md for user-facing changes
- Update relevant docs/ files for architectural changes
- Use descriptive commit messages

### Testing
- Unit tests in `src/` alongside implementation
- Integration tests in `tests/`
- Use fixtures in `tests/fixtures/` for consistent test data
- Aim for increasing coverage as per #23

## Architecture Guidelines

### Core Principles (per #37)
- **Index-time heavy**: Embed, distill, dedupe when ingesting
- **Read-time light**: Return excerpts (~400 chars), never full content by default
- **Write-time minimal**: Accept summary + tags only, not raw chat blobs
- **Structured SQL**: Fixed schema stable for NN export
- **Speed**: FTS + vector in SQLite, content-hash skip, query cache

### Module Boundaries
- `src/brain/`: Core knowledge storage and retrieval
- `src/connectors/`: Source-specific data ingestion
- `src/mcp/`: Agent interface layer
- `src/query/`: Search and ranking logic
- `src/app/`: Application orchestration

### Adding Connectors
Implement the `Connector` trait:
```rust
#[async_trait]
pub trait Connector: Send + Sync {
    fn name(&self) -> &str;
    async fn init(&mut self, config: &SourceConfig) -> Result<()>;
    async fn poll(&self) -> Result<Vec<KnowledgeAtom>>;
    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>>;
}
```

## Pull Request Process

### Before Opening a PR
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes following coding standards
4. Run full test suite: `cargo test --locked`
5. Format and lint: `cargo fmt && cargo clippy --all-targets -- -D warnings`
6. Ensure CI would pass: `cargo build --release --locked`

### PR Guidelines
- Link to related issues in the description
- Keep PRs focused and reasonably sized
- Update documentation as needed
- Add tests for new functionality
- Ensure all CI checks pass

### PR Description Template
Use the provided `.github/PULL_REQUEST_TEMPLATE.md` when opening PRs.

## Issue Guidelines

### Reporting Bugs
Use the bug report template in `.github/ISSUE_TEMPLATE/bug_report.md`
- Include environment details (OS, Rust version)
- Provide reproduction steps
- Include error messages and logs

### Feature Requests
Use the feature request template in `.github/ISSUE_TEMPLATE/feature_request.md`
- Describe the use case
- Explain why this feature would be valuable
- Consider if it fits the current roadmap (#27)

### Questions
Use the question template in `.github/ISSUE_TEMPLATE/question.md`
- Check existing issues and discussions first
- Provide context for your question

## Project Structure

```
kurultai/
├── src/
│   ├── app/          # Application orchestration
│   ├── brain/        # Core knowledge storage
│   ├── connectors/   # Source ingestion
│   ├── embed/        # Embedding logic
│   ├── mcp/          # Agent interface
│   ├── query/        # Search and ranking
│   └── store/        # SQLite storage
├── tests/
│   └── fixtures/     # Test data
├── docs/
│   ├── plans/        # Implementation plans
│   └── solutions/    # Architecture decisions
└── .github/
    └── workflows/    # CI/CD
```

## Roadmap Alignment

The project follows a phased approach (#27):
- **Phase 1**: Foundation (storage, embeddings, CLI, MCP)
- **Phase 2**: Search & Retrieval (RRF, reranking)
- **Phase 3**: Synthesis & Interface (daemon, agent capture)
- **Phase 4**: Expansion (GitHub, Dayflow connectors)
- **Phase 5**: Production (performance, shared daemon)
- **Phase 6**: Launch (release, company features)

Check if your contribution aligns with the current phase before starting work.

## Getting Help

- Check existing issues and discussions
- Review documentation in `docs/`
- Consult the upstream inspiration matrix (#40)
- Open a question issue if needed

## License

By contributing, you agree that your contributions will be licensed under the MIT License.