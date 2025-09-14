# Contributing to CardioTrust

Thank you for your interest in CardioTrust! This is primarily a personal research project where I continue to develop and refine cardiac electrophysiological simulation algorithms from my PhD work.

## What contributions are welcome?

While this is a personal project, I appreciate help with:

- **Bug reports** - If something isn't working as expected
- **Security issues** - Vulnerabilities or security concerns
- **Algorithm discussions** - Questions about the technical approach or mathematical foundations
- **Performance improvements** - Optimization suggestions that preserve research integrity
- **Documentation improvements** - Clarifications or corrections in technical documentation

## What contributions are NOT suitable?

Please avoid contributions that:

- **Alter core algorithms** - The mathematical foundations must remain consistent with published research
- **Break GPU/CPU parity** - Changes must maintain consistency between execution modes
- **Compromise research integrity** - Algorithm behavior should not change unless explicitly intended

## How to contribute

### Reporting Issues

1. Check if the issue already exists in [GitHub Issues](https://github.com/DenominatorIsZero/cardiotrust/issues)
2. Create a new issue with:
   - Clear description of the problem
   - Steps to reproduce (for bugs)
   - Expected vs actual behavior
   - Your environment (OS, GPU, Rust version) if relevant
   - Relevant log files from `./logs/` directory

### Security Vulnerabilities

Please report security issues responsibly by contacting me directly:

- **Contact**: [Erik Engelhardt on LinkedIn](https://www.linkedin.com/in/erik-engelhardt-65b1091a7/)

## Development Setup

If you want to run the project locally:

```bash
# Clone and setup
git clone https://github.com/DenominatorIsZero/cardiotrust.git
cd cardiotrust

# Install dependencies
cargo install just cargo-nextest
rustup toolchain install nightly

# Initial build and test setup
just build
just test  # This will fail the first time
just run   # Run application to generate test data
just test  # This should pass the second time
```

For all available commands, run `just` to see the help menu or check the README.

## Development Guidelines

### Code Quality
- Run `just work` before submitting changes (lint + test + benchmarks)
- Follow existing code style and patterns
- Maintain performance benchmarks - use `just bench` to verify
- Preserve algorithm correctness - test with multiple scenario types

### Testing Strategy
- Note the two-run testing requirement (see README for details)
- Visual outputs are saved to `tests/` directory for verification
- Run benchmarks to ensure performance isn't degraded
- Test both GPU and CPU execution paths when relevant

### Research Software Considerations
- **Algorithm Preservation** - Core mathematical algorithms should not change behavior
- **GPU/CPU Consistency** - Maintain identical results between execution modes
- **Performance Monitoring** - Use existing benchmarks to verify optimizations
- **Documentation** - Technical changes require corresponding documentation updates

## Code of Conduct

Please be respectful and constructive in all interactions. This is a research project with specific requirements for algorithm integrity and reproducibility.

## Architecture and Technical Details

For technical background, see:
- `docs/architecture.md` - System overview and component relationships
- Published papers (referenced in README.md) - Mathematical foundations
- `CLAUDE.md` - AI-assisted development workflow and guidelines

## Questions?

Feel free to reach out on [LinkedIn](https://www.linkedin.com/in/erik-engelhardt-65b1091a7/) if you have questions about:
- The research background or mathematical approach
- Technical implementation decisions
- How to get started with cardiac electrophysiology simulation
- Potential collaboration opportunities

I'm always happy to discuss the technical aspects of this work with fellow researchers and developers!