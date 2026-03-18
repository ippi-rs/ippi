# Contributing to IPPI

Thank you for your interest in contributing to IPPI! This document provides guidelines and instructions for contributing to the project.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Community](#community)

## 📜 Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md). We are committed to providing a friendly, safe, and welcoming environment for all contributors.

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.70 or later ([rustup](https://rustup.rs/))
- **Node.js**: 18 or later (for frontend development)
- **Git**: Latest version
- **Docker**: Optional, for containerized development

### Setting Up Development Environment

1. **Fork the repository**
   ```bash
   # Fork on GitHub, then clone your fork
    git clone https://github.com/YOUR_USERNAME/ippi.git
    cd ippi
   ```

2. **Set up upstream remote**
   ```bash
    git remote add upstream https://github.com/ippi-rs/ippi.git
   ```

3. **Install dependencies**
   ```bash
   # Rust dependencies (will be downloaded by Cargo)
   cargo fetch
   
   # Frontend dependencies
   cd frontend
   npm install
   cd ..
   ```

4. **Build the project**
   ```bash
   # Development build
   cargo build
   
   # Build with frontend embedded
   cargo build --features frontend-embedded
   ```

5. **Run tests**
   ```bash
   cargo test
   ```

### Development Scripts

We provide several scripts in the `scripts/` directory:

```bash
# Run development server with hot reload
./scripts/dev.sh

# Run tests
./scripts/test.sh

# Format code
./scripts/fmt.sh

# Lint code
./scripts/lint.sh

# Build for Raspberry Pi
./scripts/build-pi-zero.sh
```

## 🔄 Development Workflow

### 1. Find an Issue

Check our [GitHub Issues](https://github.com/ippi-rs/ippi/issues) for tasks:
- **Good first issues**: Labeled `good-first-issue`
- **Help wanted**: Labeled `help-wanted`
- **Bugs**: Labeled `bug`
- **Features**: Labeled `enhancement`

If you want to work on something not listed, please open an issue first to discuss.

### 2. Create a Branch

```bash
# Sync with upstream
git fetch upstream
git checkout main
git rebase upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test improvements
- `chore/` - Maintenance tasks

### 3. Make Changes

Follow our [Code Style](#code-style) guidelines. Make small, focused commits:

```bash
# Add changes
git add .

# Commit with descriptive message
git commit -m "feat: add video streaming support

- Implement WebRTC video bridge
- Add frame capture from KVM
- Support VP8 and H.264 codecs

Fixes #123"
```

Commit message format (Conventional Commits):
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `style:` Formatting
- `refactor:` Code refactoring
- `test:` Adding tests
- `chore:` Maintenance

### 4. Keep Branch Updated

```bash
# Regularly sync with upstream
git fetch upstream
git rebase upstream/main
```

### 5. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration

# Run with frontend
cargo test --features frontend-embedded

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

## 🎨 Code Style

### Rust Code

We use `rustfmt` and `clippy` to maintain code quality:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

#### Key Guidelines:

1. **Imports**: Group and sort imports
   ```rust
   // std imports first
   use std::collections::HashMap;
   use std::path::Path;
   
   // external crates
   use anyhow::{Context, Result};
   use tokio::sync::Mutex;
   
   // internal modules
    use crate::error::IppiError;
   use crate::web::routes;
   ```

2. **Error Handling**: Use `anyhow` and `thiserror`
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum VmError {
       #[error("VM not found: {0}")]
       NotFound(String),
       
       #[error("KVM error: {0}")]
       Kvm(#[from] kvm_ioctls::Error),
   }
   ```

3. **Async/Await**: Use `tokio` and proper error propagation
   ```rust
   pub async fn start_vm(&self, config: &VmConfig) -> Result<VmHandle> {
       let vm = self.create_vm(config).await.context("Failed to create VM")?;
       vm.start().await.context("Failed to start VM")?;
       Ok(vm)
   }
   ```

4. **Documentation**: Document public API
   ```rust
   /// Starts a virtual machine with the given configuration.
   ///
   /// # Arguments
   /// * `config` - VM configuration including memory, CPUs, and devices
   ///
   /// # Returns
   /// A handle to the running VM that can be used to control it.
   ///
   /// # Errors
   /// Returns an error if the VM cannot be created or started.
   pub async fn start_vm(&self, config: &VmConfig) -> Result<VmHandle> {
       // ...
   }
   ```

### Frontend Code (Svelte/TypeScript)

1. **TypeScript**: Use strict mode and explicit types
2. **Components**: Keep components small and focused
3. **Styling**: Use CSS variables for theming
4. **State Management**: Use Svelte stores for global state

### Configuration Files

- Use YAML for user-facing configuration
- Use TOML for internal configuration
- Include comments explaining options

## 🧪 Testing

### Test Structure

```
tests/
├── unit/           # Unit tests
├── integration/    # Integration tests
├── e2e/           # End-to-end tests
└── benches/       # Benchmarks
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    
    #[test]
    fn test_vm_creation() {
        // Unit test
    }
    
    #[tokio::test]
    async fn test_async_operation() {
        // Async test
    }
    
    #[rstest]
    #[case(512, 1)]
    #[case(1024, 2)]
    fn test_with_parameters(#[case] memory: u64, #[case] cpus: u32) {
        // Parameterized test
    }
}
```

### Test Guidelines

1. **Unit Tests**: Test individual functions in isolation
2. **Integration Tests**: Test component interactions
3. **E2E Tests**: Test complete workflows
4. **Mocking**: Use mocks for external dependencies
5. **Fixtures**: Use fixtures for test data

## 📚 Documentation

### Documentation Types

1. **Code Documentation**: Rust doc comments
2. **API Documentation**: OpenAPI/Swagger specs
3. **User Guides**: Step-by-step tutorials
4. **Architecture Docs**: System design documents
5. **Deployment Guides**: Production setup instructions

### Writing Documentation

```markdown
# Title

Brief description.

## Section

Detailed information.

### Subsection

More specific details.

```rust
// Code examples
```

**Note**: Important information.

> Tip: Helpful suggestion.
```

## 🔀 Pull Request Process

### 1. Create Pull Request

1. Push your branch to your fork
   ```bash
   git push origin feature/your-feature-name
   ```

2. Open a Pull Request on GitHub
   - Use the PR template
   - Link related issues
   - Describe changes clearly

### 2. PR Review

- **Automated Checks**: CI must pass
- **Code Review**: At least one maintainer must approve
- **Changes Requested**: Address all review comments
- **Update PR**: Push updates to the same branch

### 3. Merge

- **Squash and Merge**: For feature branches
- **Rebase and Merge**: For clean history
- **Merge Commit**: For collaborative branches

### PR Checklist

- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CI passes
- [ ] Changes are backward compatible
- [ ] Issue linked in description

## 🏛️ Project Structure

### Core Components

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library exports
├── config/              # Configuration
├── error/               # Error types
├── web/                 # Web server
├── kvm/                 # KVM hypervisor
├── p2p/                 # P2P networking
├── webrtc/              # WebRTC bridge
├── cloud_init/          # Cloud-init service
├── hardware/            # Hardware abstraction
└── utils/               # Utilities
```

### Adding New Features

1. **Design**: Document architecture in `docs/design/`
2. **Implementation**: Create module in appropriate directory
3. **Testing**: Add comprehensive tests
4. **Documentation**: Update relevant docs
5. **Examples**: Add usage examples

## 🐛 Reporting Bugs

### Bug Report Template

```markdown
## Description
Brief description of the issue.

## Steps to Reproduce
1. Step 1
2. Step 2
3. Step 3

## Expected Behavior
What should happen.

## Actual Behavior
What actually happens.

## Environment
- IPPI version:
- OS and version:
- Hardware (Pi model):
- Rust version:
- Node.js version:

## Logs
Relevant log output.

## Additional Context
Screenshots, configuration, etc.
```

## 💡 Feature Requests

### Feature Request Template

```markdown
## Problem Statement
Describe the problem this feature would solve.

## Proposed Solution
Describe how the feature should work.

## Alternatives Considered
Other ways to solve the problem.

## Additional Context
Use cases, examples, etc.
```

## 👥 Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and discussions
- **Discord**: Real-time chat ([invite link](https://discord.gg/ippi))
- **Monthly Calls**: Community meetings

### Getting Help

1. Check [documentation](https://ippi.rs/docs)
2. Search existing issues and discussions
3. Ask in Discord or GitHub Discussions
4. Open an issue if it's a bug

### Recognition

Contributors are recognized in:
- GitHub contributors list
- Release notes
- Project documentation
- Community announcements

## 📝 License

By contributing, you agree that your contributions will be licensed under the project's [GPL-3.0 license](LICENSE).

## 🙏 Thank You!

Thank you for contributing to IPPI! Your efforts help make lightweight, accessible KVM-over-IP available to everyone.

---

*This document is adapted from [Contributor Covenant](https://www.contributor-covenant.org) and other open source projects.*