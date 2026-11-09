For the Persian version, see [CONTRIBUTING-FA.md](https://github.com/IntelligentFalcon/falconlauncher/blob/master/CONTRIBUTING-FA.md).

# Contributing to Falcon Launcher

First of all, thank you for considering contributing to **Falcon Launcher**! Every contribution, whether it's a bug fix, a new feature, an improvement, or documentation, is appreciated.

Falcon Launcher is an open-source Minecraft launcher built with Rust and Tauri. Our goal is to provide a free, reliable, and user-friendly launcher, with particular attention to the needs of the Iranian Minecraft community.

This document explains how to contribute to the project and the conventions we follow.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contribution Guidelines](#contribution-guidelines)
- [Coding Conventions](#coding-conventions)
- [Commit Message Convention](#commit-message-convention)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [License](#license)

## Getting Started

Before making any changes, please:

1. Check existing issues and pull requests to avoid duplicating ongoing work.
2. For major changes or new features, discuss your proposal with the maintainers before starting implementation.
3. Make sure you understand the existing codebase and its architecture.

If you're unsure about something, feel free to contact the maintainers through the project's communication channels.

## Development Setup

### Prerequisites

Make sure you have the following tools installed:

- **Git** – Version control.
- **Node.js** – JavaScript runtime.
- **pnpm** – Package manager.
- **Rust** – Stable Rust toolchain.
- **Tauri prerequisites** – Platform-specific dependencies required by Tauri.

Refer to the official [Tauri documentation](https://v2.tauri.app/start/prerequisites/) for platform-specific requirements.

### Clone the Repository

Fork the repository first, then clone your fork:

```bash
git clone https://github.com/<your-username>/falconlauncher.git
cd falconlauncher
```

Add the original repository as an upstream remote:

```bash
git remote add upstream https://github.com/IntelligentFalcon/falconlauncher.git
```

### Install Dependencies

```bash
pnpm install
```

### Run in Development Mode

```bash
pnpm tauri dev
```

### Build the Application

```bash
pnpm tauri build
```

For additional commands, check the scripts defined in `package.json`.

## Contribution Guidelines

We welcome different types of contributions, including:

- **Bug fixes:** Resolve existing issues and unexpected behavior.
- **Features:** Implement useful functionality that aligns with the project's goals.
- **Performance:** Improve resource usage, responsiveness, and overall performance.
- **UI/UX:** Improve the interface, accessibility, and user experience.
- **Documentation:** Improve existing documentation or add missing explanations.
- **Refactoring:** Improve code quality, readability, and maintainability.
- **Localization:** Improve translations and support for additional languages.

A few things to keep in mind:

- Keep changes focused. Avoid combining unrelated modifications in a single pull request.
- Avoid introducing unnecessary dependencies.
- Don't commit generated files, build artifacts, or local configuration files unless explicitly required.
- Don't include personal credentials, API keys, access tokens, or other sensitive information.
- Make sure your changes don't introduce regressions in existing functionality.
- Consider compatibility with different operating systems and Minecraft versions when applicable.

## Coding Conventions

Please follow the existing conventions and patterns used throughout the codebase.

### Rust

- Write clean, readable, and idiomatic Rust code.
- Prefer clear and descriptive names for variables, functions, and types.
- Handle errors properly instead of silently ignoring them.
- Avoid unnecessary cloning and allocations.
- Keep functions focused on a single responsibility.
- Use appropriate types instead of relying on loosely defined values.
- Run `cargo fmt` before submitting your changes.

### TypeScript

- Use TypeScript instead of introducing new JavaScript files where possible.
- Prefer explicit types and avoid unnecessary use of `any`.
- Follow the existing component and state-management patterns.
- Keep components reusable and avoid unnecessary complexity.
- Use the existing UI components and utilities whenever possible.

### General

- Follow the existing project structure.
- Keep naming consistent with the surrounding code.
- Avoid unnecessary comments; write comments when they clarify non-obvious behavior.
- Make sure new functionality works with the existing application architecture.

## Commit Message Convention

We follow a **Conventional Commits-inspired** commit message convention.

Every commit message **must start with a valid type prefix**, followed by a colon (`:`) and a short description.

### Format

```text
<type>: <description>
```

### Available Types

| Prefix | Purpose |
|---|---|
| `feat:` | Introduces a new feature or functionality. |
| `fix:` | Fixes a bug or unexpected behavior. |
| `style:` | Changes formatting or visual styling without changing functionality. |
| `refactor:` | Refactors existing code without changing its behavior. |
| `perf:` | Improves application performance. |
| `docs:` | Adds or updates documentation. |
| `test:` | Adds or modifies tests. |
| `build:` | Changes build configuration or dependencies. |
| `ci:` | Changes CI/CD workflows and configurations. |
| `chore:` | Performs maintenance tasks that don't fit other categories. |
| `revert:` | Reverts a previous commit. |

### Examples

**Good commit messages:**

```text
feat: add Fabric mod loader support
fix: resolve Microsoft authentication failure
style: improve sidebar spacing
refactor: simplify download manager logic
perf: optimize Minecraft version loading
docs: update installation instructions
test: add tests for profile management
build: update Tauri dependencies
ci: configure automated Rust checks
chore: update project dependencies
revert: revert mod loader selection changes
```

**Incorrect commit messages:**

```text
Added a new feature
Fixed a bug
Update files
changes
WIP
```

These messages are not accepted because they don't follow the required prefix convention.

### Additional Notes

- Always use lowercase prefixes.
- Keep the description short and meaningful.
- Write descriptions in English.
- Use the imperative form when possible (e.g., `add`, `fix`, `update`).
- Avoid vague descriptions such as `update stuff` or `fix things`.
- Use the most appropriate prefix for the actual purpose of the commit.

For changes that introduce breaking functionality or compatibility changes, append `!` to the type:

```text
feat!: redesign profile configuration format
```

## Pull Request Guidelines

Before submitting a pull request:

1. Make sure your changes are based on the latest version of the target branch.
2. Test your changes locally.
3. Review your code and remove unnecessary modifications.
4. Ensure your commit messages follow the required convention.
5. Update documentation if your changes introduce new functionality or modify existing behavior.

### Creating a Pull Request

1. Push your changes to your fork.
2. Open a pull request against the appropriate branch of the original repository.
3. Provide a clear title describing your changes.
4. Explain what you've changed and why.
5. Link any related issues.
6. Include screenshots or recordings when your changes affect the user interface.

### Pull Request Description

A good pull request should explain:

- **What:** What does this pull request change?
- **Why:** What problem does it solve or what improvement does it introduce?
- **How:** How was the change implemented?
- **Testing:** How did you verify that everything works?

Please keep pull requests focused and reasonably sized to make reviewing easier.

## Reporting Bugs

If you encounter a bug, please check existing issues before opening a new one.

When reporting a bug, include as much relevant information as possible:

- A clear and descriptive title.
- Steps to reproduce the issue.
- Expected behavior.
- Actual behavior.
- Operating system and version.
- Falcon Launcher version.
- Relevant logs, error messages, or screenshots.

Avoid reporting multiple unrelated bugs in a single issue.

## Suggesting Features

Feature suggestions are welcome!

When proposing a new feature, please explain:

- What the feature does.
- Why it would be useful.
- How you imagine it working.
- Any relevant examples or references.

Features should generally align with Falcon Launcher's purpose and improve the experience of its users.

For significant architectural changes, please discuss the idea with the maintainers before implementing it.

## License

By contributing to Falcon Launcher, you agree that your contributions will be distributed under the project's existing license, as specified in the `LICENSE` file.

Please make sure you have the right to submit any code, assets, or other materials included in your contribution. Third-party dependencies and assets may have separate licensing requirements.

---

Thank you for helping make Falcon Launcher better! ❤️

Every contribution matters, and we appreciate your time and effort.

**Happy coding!** 🚀
