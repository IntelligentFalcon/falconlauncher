---
name: error-handling
description: Guidelines on how errors are handled in this project.
---

# Error Handling Guidelines

This project uses a unified error handling strategy across the Rust backend and the frontend. When writing code, adhere to the following rules:

## 1. Rust Backend (Commands)

- **Simple Errors (Error Codes)**:
  - For predictable, easily categorizable errors (e.g., validation failed, not found, unauthorized), use custom Rust enums (e.g., using `thiserror`).
  - These should map or serialize into standardized string error codes that the frontend can easily switch on.
- **Complicated Errors**:
  - For unexpected or complex errors (e.g., database connection failure, OS-level errors, third-party API issues), provide rich context.
  - Return detailed error messages, wrapped via `anyhow` or comprehensive custom error structs, so the frontend can surface meaningful diagnostics (using `onError`).
- **Return Type**: Always return a `Result<T, AppError>` from Tauri commands instead of panicking.

## 2. Frontend (React / Shadcn)

- **Simple Errors**: Check the returned error code from the backend and handle it inline or by displaying a specific localized message.
- **Complicated Errors (`onError`)**: For complex errors, route them through a global or component-specific `onError` handler that can display a toast, a dialog, or a detailed error state to the user.
- **Queries (`useBackend`)**:
  - When fetching data via the `useBackend` hook, if the backend returns an error (simple or complicated), the query should fail gracefully.
  - The UI must catch this and render the `Empty` component to indicate the lack of data or the failure state, rather than crashing or showing a blank screen.
