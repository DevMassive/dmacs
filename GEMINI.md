# Gemini Development Assistant Instructions

This file guides the Gemini AI assistant's work on the `dmacs` project.

## Project Goal

To build a simple, functional text editor in Rust.

## Development Workflow

1.  **Understand the Goal:** Before making changes, ensure you understand the user's request.
2.  **Test First:** Always start by running `cargo test` to ensure the current state is stable.
3.  **Implement and Refactor:** Implement new features or fix bugs as requested.
4.  **Update this File:** Keep this `GEMINI.md` file updated with the current task or major changes you are working on.
5.  **Commit Regularly:** This is a Git repository. After a logical set of changes, create a commit with a clear and descriptive message. Propose the commit message to the user for approval.

## Current Task

*   Initial setup and project analysis.

## Key Files

*   `src/lib.rs`: Core editor logic (`Editor` and `Document` structs).
*   `src/bin/dmacs.rs`: Main application entry point.
*   `tests/editor_test.rs`: Unit and integration tests.
*   `Cargo.toml`: Project dependencies and metadata.
