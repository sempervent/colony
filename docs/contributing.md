# Contributing to Colony Simulator

Thank you for your interest in contributing to the Colony Simulator! This guide will help you get started with contributing to the project.

## 🤝 How to Contribute

### Types of Contributions

We welcome various types of contributions:

- **Bug Reports**: Report issues and bugs
- **Feature Requests**: Suggest new features and improvements
- **Code Contributions**: Submit code changes and improvements
- **Documentation**: Improve documentation and guides
- **Testing**: Help test the software and report issues
- **Community**: Help other users and contribute to discussions

### Getting Started

1. **Fork the Repository**: Create your own fork of the project
2. **Clone Your Fork**: `git clone https://github.com/your-username/colony.git`
3. **Create a Branch**: `git checkout -b feature/your-feature-name`
4. **Make Changes**: Implement your changes
5. **Test Your Changes**: Ensure all tests pass
6. **Submit a Pull Request**: Create a PR with your changes

## 📋 Development Setup

### Prerequisites

- **Rust** 1.70+ with Cargo
- **Git** for version control
- **Make** (optional, for build automation)

### Setup Instructions

```bash
# Clone the repository
git clone https://github.com/colony-simulator/colony.git
cd colony

# Install development dependencies
make install-deps

# Build the project
make build

# Run tests
make test

# Run linting
make lint

# Format code
make fmt
```

### Project Structure

```
colony/
├── crates/                 # Rust crates
│   ├── colony-core/       # Core simulation engine
│   ├── colony-desktop/    # Desktop GUI application
│   ├── colony-headless/   # Headless server
│   ├── colony-mod-cli/    # Mod development CLI
│   ├── colony-modsdk/     # Modding SDK
│   └── xtask/             # Build and verification tools
├── docs/                  # Documentation
├── mods/                  # Example mods
├── scripts/               # Build and utility scripts
└── tests/                 # Integration tests
```

## 🎯 Development Workflow

### Branch Naming

Use descriptive branch names:

- `feature/description` - New features
- `bugfix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring
- `test/description` - Test improvements

### Commit Messages

Follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

#### Types
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions or changes
- `chore`: Build process or auxiliary tool changes

#### Examples
```
feat(core): add thermal throttling system
fix(desktop): resolve UI rendering issue
docs(api): update WASM ABI documentation
test(integration): add E2E tests for M7 features
```

### Pull Request Process

1. **Create a Branch**: Use a descriptive branch name
2. **Make Changes**: Implement your changes with tests
3. **Run Tests**: Ensure all tests pass locally
4. **Update Documentation**: Update relevant documentation
5. **Create PR**: Submit a pull request with a clear description
6. **Address Feedback**: Respond to review comments
7. **Merge**: Once approved, your changes will be merged

### PR Description Template

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
```

## 🧪 Testing Guidelines

### Test Types

#### Unit Tests
- Test individual functions and methods
- Use `#[cfg(test)]` modules
- Aim for high test coverage
- Test edge cases and error conditions

#### Integration Tests
- Test component interactions
- Use `tests/` directory
- Test complete workflows
- Verify API contracts

#### End-to-End Tests
- Test complete user workflows
- Use headless server for API testing
- Test mod functionality
- Verify performance requirements

### Running Tests

```bash
# Run all tests
cargo test --workspace --all-features

# Run specific test suite
cargo test --package colony-core --test unit_tests

# Run integration tests
cargo test --package colony-headless --test e2e_integration

# Run benchmarks
cargo bench --workspace --all-features
```

### Test Coverage

- **Target**: 95%+ test coverage
- **Critical Paths**: 100% coverage for core systems
- **Edge Cases**: Test boundary conditions
- **Error Handling**: Test error conditions and recovery

## 📝 Code Style Guidelines

### Rust Style

Follow standard Rust conventions:

- **Formatting**: Use `cargo fmt`
- **Linting**: Use `cargo clippy`
- **Documentation**: Document public APIs
- **Naming**: Use descriptive names

### Code Organization

- **Modules**: Organize code into logical modules
- **Functions**: Keep functions focused and small
- **Comments**: Explain complex logic
- **Documentation**: Document public interfaces

### Performance

- **Efficiency**: Write efficient code
- **Memory**: Minimize memory allocations
- **CPU**: Optimize hot paths
- **Benchmarks**: Add benchmarks for critical code

## 🔍 Code Review Process

### Review Criteria

- **Functionality**: Does the code work as intended?
- **Style**: Does it follow project conventions?
- **Testing**: Are there adequate tests?
- **Documentation**: Is documentation updated?
- **Performance**: Are there performance implications?

### Review Process

1. **Automated Checks**: CI runs tests and linting
2. **Human Review**: Team members review the code
3. **Feedback**: Reviewers provide constructive feedback
4. **Iteration**: Address feedback and make changes
5. **Approval**: Once approved, the PR is merged

### Review Guidelines

- **Be Constructive**: Provide helpful feedback
- **Be Specific**: Point out specific issues
- **Be Respectful**: Maintain a positive tone
- **Be Thorough**: Check all aspects of the code

## 🐛 Bug Reports

### Reporting Bugs

Use the GitHub issue template:

```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g., Ubuntu 22.04]
- Rust Version: [e.g., 1.70.0]
- Colony Version: [e.g., 0.9.0-rc1]

## Additional Context
Any additional information
```

### Bug Triage

- **Severity**: Critical, High, Medium, Low
- **Priority**: P0, P1, P2, P3
- **Labels**: Use appropriate labels
- **Assignees**: Assign to appropriate team members

## 💡 Feature Requests

### Suggesting Features

Use the GitHub issue template:

```markdown
## Feature Description
Clear description of the feature

## Use Case
Why is this feature needed?

## Proposed Solution
How should this feature work?

## Alternatives
Other solutions considered

## Additional Context
Any additional information
```

### Feature Evaluation

- **Alignment**: Does it align with project goals?
- **Complexity**: How complex is the implementation?
- **Impact**: What is the user impact?
- **Resources**: What resources are required?

## 📚 Documentation

### Documentation Types

- **API Documentation**: Document public APIs
- **User Guides**: Help users use the software
- **Developer Guides**: Help developers contribute
- **Architecture Docs**: Explain system design

### Documentation Standards

- **Clarity**: Write clear, concise documentation
- **Completeness**: Cover all necessary information
- **Examples**: Provide code examples
- **Accuracy**: Keep documentation up to date

### Contributing to Documentation

- **Markdown**: Use Markdown for documentation
- **Structure**: Follow existing documentation structure
- **Links**: Use relative links for internal references
- **Images**: Optimize images for web display

## 🎨 Modding Contributions

### Creating Mods

- **Follow Guidelines**: Use the modding guidelines
- **Test Thoroughly**: Test your mods extensively
- **Document**: Provide clear documentation
- **Share**: Share your mods with the community

### Mod Quality Standards

- **Functionality**: Mods should work as intended
- **Performance**: Mods should not impact performance
- **Security**: Mods should be secure
- **Compatibility**: Mods should be compatible

## 🏷️ Release Process

### Version Numbering

We use [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Process

1. **Feature Freeze**: Stop adding new features
2. **Testing**: Comprehensive testing phase
3. **Release Candidate**: Create RC for testing
4. **Final Testing**: Community testing of RC
5. **Release**: Create final release
6. **Documentation**: Update release notes

## 🤝 Community Guidelines

### Code of Conduct

We follow the [Contributor Covenant](https://www.contributor-covenant.org/):

- **Be Respectful**: Treat everyone with respect
- **Be Inclusive**: Welcome people of all backgrounds
- **Be Constructive**: Provide helpful feedback
- **Be Professional**: Maintain professional behavior

### Communication

- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For general discussions
- **Discord**: For real-time community chat
- **Email**: For sensitive or private matters

## 🎯 Getting Help

### Resources

- **Documentation**: Check the documentation first
- **GitHub Issues**: Search existing issues
- **Discord**: Ask questions in the community
- **Email**: Contact maintainers directly

### Mentorship

- **New Contributors**: We provide mentorship for new contributors
- **Pair Programming**: We encourage pair programming sessions
- **Code Reviews**: We provide detailed code reviews
- **Feedback**: We provide constructive feedback

## 🏆 Recognition

### Contributor Recognition

- **Contributors List**: All contributors are recognized
- **Release Notes**: Contributors are mentioned in release notes
- **Hall of Fame**: Outstanding contributors are recognized
- **Swag**: Contributors may receive project swag

### Contribution Levels

- **Contributor**: Any contribution to the project
- **Maintainer**: Regular contributors with merge rights
- **Core Team**: Long-term contributors with project ownership

## 📞 Contact

### Maintainers

- **Project Lead**: [Your Name] - [email@example.com]
- **Technical Lead**: [Technical Lead Name] - [tech@example.com]
- **Community Manager**: [Community Manager Name] - [community@example.com]

### Communication Channels

- **GitHub**: [colony-simulator/colony](https://github.com/colony-simulator/colony)
- **Discord**: [Join our community](https://discord.gg/colony-simulator)
- **Email**: [contact@colony-simulator.com](mailto:contact@colony-simulator.com)

---

**Thank you for contributing to the Colony Simulator! Together, we're building something amazing.** 🏭✨
