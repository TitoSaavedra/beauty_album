# BDO Beauty Album Desktop Engine

Desktop architecture for the Black Desert Online Beauty Album viewer, powered by Tauri 2.0 and Rust.

This application replaces the previous local FastAPI architecture with native desktop IPC communication through Tauri Commands, isolating rendering from data processing and filesystem operations.

---

# Core Principles

- Keep implementations minimal and focused.
- Prefer extending existing systems over introducing new abstractions.
- Maintain strict separation between frontend, IPC layer, and business logic.
- Avoid speculative architecture.
- Optimize for maintainability and desktop responsiveness.
- Prioritize consistency over personal preference.
- Do not build or Test

---

# AI Assistant Behavior Rules

## General Behavior

- Keep responses concise.
- Implement only the requested change.
- Do not rewrite unrelated code.
- Do not add explanations unless explicitly requested.
- Do not generate tests unless explicitly requested.
- Do not generate documentation unless explicitly requested.
- Do not introduce new dependencies unless required.
- Do not rename files, folders, functions, or modules without necessity.
- Do not move code between modules unless explicitly requested.
- Do not add placeholder implementations.
- Do not leave TODO comments.

## Refactoring Restrictions

- Avoid large-scale refactors.
- Avoid formatting-only changes.
- Avoid modifying import ordering unless required.
- Avoid changing public interfaces unless necessary.
- Avoid creating abstractions for single-use logic.
- Prefer incremental modifications over full rewrites.

## Code Style

- Write production-ready code.
- Prefer explicit and readable logic.
- Use descriptive naming.
- Keep functions focused and small.
- Avoid deeply nested conditionals.
- Avoid magic values.
- Prefer early returns.
- Prefer composition over inheritance.
- Keep modules single-purpose.

---

# Architecture Rules

## Layer Separation

Frontend:
- Rendering
- UI state
- User interaction
- Visual composition

Tauri Commands:
- IPC boundary only
- Request validation
- Response mapping
- Error translation

Rust Services:
- Business logic
- File parsing
- Data transformation
- Caching
- Filesystem operations

Persistence Layer:
- File IO
- Local storage
- Asset indexing

## Forbidden Patterns

- Business logic inside UI components.
- Business logic inside Tauri commands.
- Direct filesystem access from frontend code.
- Shared mutable global state unless required.
- God modules handling unrelated responsibilities.
- Circular module dependencies.
- Frontend-specific logic inside Rust services.

---

# Tauri 2.0 Rules

## Commands

- Keep commands thin.
- Commands must delegate logic into services.
- Commands must return typed responses.
- Avoid unnecessary IPC roundtrips.
- Prefer batching over repeated frontend invocations.

## Events

- Use events only for async state notifications.
- Avoid event spam.
- Prefer direct command responses when possible.

## State

- Use managed state only when ownership is truly shared.
- Avoid unnecessary Arc<Mutex<T>> usage.
- Prefer immutable data flow where possible.

## Paths and Filesystem

- Use PathBuf and Path consistently.
- Use native filesystem utilities.
- Normalize paths before persistence.
- Avoid string-based path manipulation.

---

# Rust Backend Standards

## Language

- Use Rust 2021 edition.
- Follow idiomatic Rust patterns.
- Prefer Result<T, E> propagation.
- Avoid unwrap and expect outside startup/bootstrap code.
- Prefer pattern matching over chained conditionals.

## Error Handling

- Use typed errors.
- Return meaningful error messages.
- Avoid panic-based flow control.
- Propagate recoverable failures cleanly.

## Async

- Use async only for IO-bound operations.
- Avoid unnecessary async propagation.
- Avoid spawning detached tasks unless required.

## Memory and Performance

- Prefer references over cloning.
- Avoid unnecessary allocations.
- Minimize lock duration.
- Avoid excessive Arc usage.
- Reuse parsed data when possible.

## Module Structure

Example:

src-tauri/src/
- commands/
- services/
- models/
- repositories/
- state/
- utils/
- errors/

---

# Frontend Rules

## UI Architecture

- Keep components presentation-focused.
- Extract reusable UI primitives only after repetition appears.
- Avoid oversized components.
- Prefer feature-based organization.

## State Management

- Avoid duplicated state.
- Avoid unnecessary reactive subscriptions.
- Keep derived state computed.
- Minimize IPC-triggered rerenders.

## Styling

- Use modular SCSS architecture.
- Combine SCSS structure with Tailwind utility composition.
- Avoid inline styles.
- Avoid global style leakage.
- Keep design tokens centralized.

Example:

src/
- features/
- components/
- layouts/
- services/
- stores/
- styles/
- tauri/

---

# Performance Rules

## IPC

- Minimize frontend/backend chatter.
- Avoid polling unless explicitly required.
- Batch filesystem reads when possible.
- Cache stable metadata.

## Rendering

- Avoid unnecessary rerenders.
- Virtualize large lists when needed.
- Prefer lazy asset loading.
- Avoid expensive computed bindings during render cycles.

## File Processing

- Stream large files when appropriate.
- Avoid loading unnecessary assets into memory.
- Reuse decoded data structures.

---

# Dependency Rules

- Prefer standard library utilities first.
- Avoid large dependencies for small problems.
- Do not introduce crates without clear justification.
- Keep dependency footprint minimal.

---

# Naming Conventions

## Rust

- snake_case for files and modules
- PascalCase for structs and enums
- camelCase only when interoperating with frontend payloads

## Frontend

- PascalCase for components
- camelCase for variables and functions
- kebab-case for style modules when applicable

---

# Output Expectations

When generating code:
- Return only the necessary changes.
- Preserve existing architecture.
- Match surrounding style conventions.
- Keep implementations concise.
- Avoid unrelated cleanup.