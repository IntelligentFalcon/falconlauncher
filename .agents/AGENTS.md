# AI Agent Guidelines

Welcome to the Falcon Launcher project! As an AI agent working on this codebase, you must always adhere to the project's established conventions and guidelines. 

## Skills and Rules
Before beginning any significant task, you are **required** to check the skills defined in the `.agents/skills/` directory for specific instructions.

Currently active skills include:
- **Error Handling**: When writing or modifying backend Rust code or frontend error boundaries, you **must** read and follow the instructions in `.agents/skills/error_handling/SKILL.md`. This includes rules on using `thiserror`/`anyhow` on the backend, always returning `Result<T, AppError>` from Tauri commands, and rendering the `Empty` component or `onError` handlers on the frontend.
- **Shadcn UI**: When writing or modifying frontend components, you **must** read and follow `.agents/skills/shadcn/SKILL.md` to ensure proper usage of the Shadcn component library.

## General AI Directives
1. **Always read relevant skills**: If a task involves error handling, API responses, or frontend components, read the corresponding SKILL.md file before writing code.
2. **Do not hallucinate patterns**: Stick to the patterns defined in the skills (e.g. `useBackend` error handling) instead of inventing new ways to handle errors or style components.
3. **Consistency**: Maintain consistency with the surrounding codebase.

Failure to follow these skills will result in architectural inconsistency.
