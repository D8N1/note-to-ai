# Project Status: WIP

This project is under active development. Major areas still in progress:

- Semantic query embeddings: use real query vectors (currently placeholder)
- Config-driven embeddings: read model selection from `config.toml`
- Block embeddings: generate from parsed blocks and store positions/types
- Watcher: file change detection for live re-indexing
- Tests: add minimal integration test (index temp vault → query)
- CLI polish: richer output, error messages, and help text
- Documentation: flesh out model setup, performance tips, and troubleshooting

What works now:
- Vault orchestrator indexes files and writes search + embeddings transactionally
- CLI `index` and `query` are wired to the Vault
- Hybrid text+semantic search path (semantic uses a placeholder vector for now)

If you’re evaluating: expect frequent changes; APIs may evolve. Feedback and contributions are welcome.
