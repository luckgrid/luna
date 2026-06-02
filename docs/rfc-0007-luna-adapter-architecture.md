---
title: "RFC-0007: Luna Adapter Architecture"
description: "Specification for adapter registry, adapter API, dependency delegation, ecosystem integration, and toolchain operations."
status: draft
rfc: "0007"
type: rfc
depends_on:
  - RFC-0001
  - RFC-0002
  - RFC-0005
referenced_by:
  - RFC-0008
---

## Overview

This RFC defines the Adapter System.

Adapters are the mechanism that allows Luna to remain:

```text
Language Neutral
Tool Neutral
Framework Neutral
```

while still providing a unified developer experience.

Adapters act as translation layers between:

```text
Luna Commands
```

and

```text
Ecosystem Tooling
```

Examples:

```text
luna add
luna update
luna doctor
luna create
```

↓

```text
bun add
uv add
cargo add
go get
```

Adapters are one of the most important architectural boundaries in Luna.

## Problem Statement

Without adapters, Luna becomes tightly coupled to:

```text
Bun
TypeScript
Node.js
```

which directly conflicts with Luna's vision.

Examples:

```text
TypeScript Project
Python Project
Go Project
Rust Project
```

should all feel like first-class workspace citizens.

However:

```text
Dependencies
Lockfiles
Project Metadata
Install Commands
Update Commands
```

are different across ecosystems.

Adapters solve this problem.

## Goals

### Goal 1

Abstract ecosystem operations.

### Goal 2

Maintain native tooling.

### Goal 3

Provide a unified Luna UX.

### Goal 4

Allow future ecosystems.

### Goal 5

Prevent Luna from becoming a package manager.

## Non-Goals

### Luna Does Not Resolve Dependencies

Luna never computes dependency graphs.

Examples:

```text
bun
uv
cargo
go
```

remain responsible.

### Luna Does Not Maintain Lockfiles

Examples:

```text
bun.lock
uv.lock
Cargo.lock
go.sum
```

remain ecosystem owned.

### Luna Does Not Replace Package Managers

Adapters delegate.

Adapters do not reimplement.

## Design Principles

### Principle 1

Native tooling first.

If the ecosystem already solves a problem:

```text
Use it.
```

### Principle 2

Thin abstraction.

Adapters should translate operations.

Not reimplement them.

### Principle 3

Consistent interface.

Every adapter should expose similar capabilities.

## High-Level Architecture

```text
Luna Command
      │
      ▼

Adapter Registry
      │
      ▼

Adapter
      │
      ▼

Native Tool
```

Example:

```text
luna add zod

      ↓

BunAdapter

      ↓

bun add zod
```

## Core Adapter Model

```rust
pub trait Adapter {
    fn name(&self) -> &str;

    fn install(&self);
    fn update(&self);
    fn remove(&self);
    fn doctor(&self);
}
```

## Why Traits?

Traits provide:

```text
Static Typing
Compile-Time Validation
Easy Testing
Extensibility
```

## Adapter Registry

Central adapter manager.

### Responsibilities

```text
Registration
Lookup
Resolution
Lifecycle
```

### Example

```rust
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn Adapter>>,
}
```

## Resolution

Luna resolves adapters using project metadata.

### Example

```yaml
projects:
  ui:
    language: typescript
```

↓

```text
BunAdapter
```

### Example

```yaml
projects:
  api:
    language: python
```

↓

```text
UvAdapter
```

## Adapter Categories

Adapters are grouped by ecosystem.

## TypeScript Adapter

### Initial Implementation

```text
BunAdapter
```

### Responsibilities

```text
Install Packages
Remove Packages
Update Packages
Dependency Metadata
```

### Commands

```sh
luna add zod --project ui
```

↓

```sh
bun add zod
```

## Python Adapter

### Initial Implementation

```text
UvAdapter
```

### Responsibilities

```text
Dependencies
Sync
Locking
Updates
```

### Commands

```sh
luna add pydantic
```

↓

```sh
uv add pydantic
```

## Go Adapter

### Initial Implementation

```text
GoAdapter
```

### Responsibilities

```text
Dependencies
Modules
Updates
Validation
```

### Commands

```sh
luna add gin
```

↓

```sh
go get github.com/gin-gonic/gin
```

## Rust Adapter

### Initial Implementation

```text
CargoAdapter
```

### Responsibilities

```text
Dependencies
Updates
Validation
Workspace Integration
```

### Commands

```sh
luna add clap
```

↓

```sh
cargo add clap
```

## Adapter Lifecycle

Every adapter participates in:

```text
Discovery
Validation
Execution
Reporting
```

## Discovery

Detect ecosystem metadata.

Examples:

```text
package.json
pyproject.toml
go.mod
Cargo.toml
```

## Validation

Examples:

```text
Missing Dependencies
Broken Config
Invalid Environment
```

## Execution

Delegates commands.

## Reporting

Returns structured results.

### Example

```rust
pub struct AdapterResult {
    pub success: bool,
    pub message: String,
}
```

## Project Context

Adapters always execute within project context.

### Example

```sh
luna add zod --project ui
```

Luna determines:

```text
Project
Path
Language
Adapter
```

before execution.

## Workspace Context

Some operations span multiple projects.

### Example

```sh
luna update
```

Luna executes:

```text
BunAdapter
UvAdapter
GoAdapter
CargoAdapter
```

in sequence.

## Parallel Execution

Future capability.

### Example

```sh
luna update --parallel
```

## Adapter Capabilities

Not every adapter supports every operation.

### Example

```rust
pub struct AdapterCapabilities {
    pub add: bool,
    pub remove: bool,
    pub update: bool,
    pub doctor: bool,
}
```

## Capability Discovery

Luna can query:

```rust
adapter.capabilities()
```

## Structured Output

Adapters return normalized responses.

### Example

```rust
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}
```

## Dependency Listing

Future command:

```sh
luna deps
```

Aggregates:

```text
bun
uv
cargo
go
```

results into a common format.

## Outdated Packages

Future command:

```sh
luna outdated
```

Aggregates ecosystem responses.

## Workspace Updates

Future command:

```sh
luna update
```

Delegates:

```text
bun update
uv sync
cargo update
go get -u
```

## Error Handling

Adapters should return typed errors.

### Example

```rust
enum AdapterError {
    UnsupportedOperation,
    ToolNotInstalled,
    InvalidProject,
    ExecutionFailed,
}
```

## Testing Strategy

### Unit Tests

Test:

```text
Resolution
Capabilities
Metadata Parsing
```

### Integration Tests

Test:

```text
Real Bun
Real uv
Real Cargo
Real Go
```

execution.

## Future Plugin Adapters

Not part of the current refactor.

### Goal

Support:

```text
Terraform
OpenTofu
Docker
Kubernetes
Helm
Pulumi
```

## Future Architecture

```text
plugins/
```

or:

```text
~/.luna/plugins
```

## Dynamic Adapter Loading

Future possibility:

```rust
Adapter
↓
Shared Library
↓
Runtime Registration
```

## Why Not Plugins Yet?

Plugins add:

```text
Security Concerns
Compatibility Concerns
Compatibility Concerns
```

Static adapters are simpler initially.

## Acceptance Criteria

The adapter architecture is complete when:

- Ecosystem operations are abstracted
- Luna remains package-manager agnostic
- New adapters can be added easily
- Workspace-wide commands function
- Adapter capabilities are discoverable
- Adapters remain thin wrappers

## Architectural Decision

### Decision

Use a registry-driven adapter architecture with static adapter implementations in Luna.

### Rationale

This provides a stable foundation while preserving ecosystem-native tooling and enabling future expansion.

### Consequences

#### Positive

- Language neutrality
- Consistent developer experience
- Extensible architecture
- Clear ownership boundaries

#### Tradeoffs

- Additional abstraction layer
- Adapter maintenance
- Ecosystem-specific edge cases
