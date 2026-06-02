---
title: "RFC-0005: Luna Runtime Architecture"
description: "Implementation blueprint for runtime services, service boundaries, command architecture, lifecycle, registries, and internal system design."
status: draft
rfc: "0005"
type: rfc
depends_on:
  - RFC-0001
  - RFC-0002
  - RFC-0003
referenced_by:
  - RFC-0006
  - RFC-0007
  - RFC-0008
---

## Overview

This RFC defines the internal architecture of the Luna runtime.

Previous RFCs establish:

- Why Luna exists
- What Luna owns
- How Luna integrates with Moon and Proto
- How configuration generation works

This RFC defines:

- Runtime architecture
- Internal service boundaries
- Package structure
- Data flow
- Command execution
- Generator architecture
- Adapter architecture integration
- Future daemon architecture

This document should be considered the primary implementation reference for Luna itself.

## Design Goals

### Goal 1

Maintain clear ownership boundaries.

```text
Luna     -> Workspace Runtime
Moon     -> Execution
Proto    -> Toolchains
Adapters -> Ecosystem Operations
```

### Goal 2

Keep Luna modular.

Each subsystem should be independently testable.

### Goal 3

Support future growth.

Future features:

```text
Daemon Mode
Plugin System
Remote Registry
Template Marketplace
Workspace Analytics
```

should not require redesign.

### Goal 4

Remain fast.

Workspace operations should be optimized for:

```text
Discovery
Generation
Validation
Synchronization
```

## High-Level Architecture

```text
CLI
 │
 ▼
Runtime
 │
 ├─ Workspace Service
 ├─ Discovery Service
 ├─ Generator Service
 ├─ Validation Service
 ├─ Adapter Service
 └─ Execution Service
        │
        ▼
Moon / Proto / Adapters
```

## Core Runtime Layers

### Layer 1

CLI Layer

Responsibilities:

- Argument parsing
- Help output
- Command routing

Examples:

```sh
luna sync
luna doctor
luna projects
```

### Layer 2

Runtime Layer

Responsibilities:

- Service initialization
- Configuration loading
- Dependency injection

### Layer 3

Domain Layer

Responsibilities:

- Workspace model
- Project model
- Registry model
- Configuration model

### Layer 4

Infrastructure Layer

Responsibilities:

- Filesystem
- Process execution
- Serialization
- Template rendering

## Proposed Crate Structure

```text
crates/
└── luna-cli/
    ├── Cargo.toml
    └── src/
```

### Source Layout

```text
src/
├── main.rs
├── cli.rs
├── runtime.rs
│
├── workspace/
├── discovery/
├── generators/
├── validators/
├── adapters/
├── templates/
├── execution/
│
├── commands/
└── infrastructure/
```

## Main Entry Point

```rust
fn main() {
    let cli = Cli::parse();

    Runtime::new()
        .execute(cli);
}
```

Responsibilities:

- Parse arguments
- Bootstrap runtime
- Execute command

Nothing else.

## Runtime

The runtime owns service registration.

### Responsibilities

```text
Workspace Service
Discovery Service
Generator Service
Validator Service
Adapter Registry
Process Runner
```

### Example

```rust
pub struct Runtime {
    workspace: WorkspaceService,
    discovery: DiscoveryService,
    generators: GeneratorRegistry,
    validators: ValidatorRegistry,
    adapters: AdapterRegistry,
}
```

## Workspace Service

The central service.

### Responsibilities

```text
Load luna.yml
Load workspace metadata
Build workspace graph
Provide project lookups
```

### Example API

```rust
workspace.projects()
workspace.project("ui")
workspace.root()
```

## Workspace Model

Internal representation.

### Workspace

```rust
pub struct Workspace {
    pub name: String,
    pub root: PathBuf,
    pub projects: Vec<Project>,
}
```

### Project

```rust
pub struct Project {
    pub id: String,
    pub path: PathBuf,
    pub language: Language,
    pub framework: Option<String>,
}
```

## Discovery Service

Responsible for locating projects.

### Discovery Order

1. luna.yml
2. Moon
3. Filesystem

### Responsibilities

```text
Project Discovery
Workspace Discovery
Manifest Discovery
```

### Example

```rust
discovery.discover_projects()
```

Returns:

```rust
Vec<Project>
```

## Generator Service

Responsible for generated files.

### Responsibilities

```text
package.json
tsconfig.json
workspace references
moon metadata
```

### Registry Pattern

```rust
GeneratorRegistry
```

Contains:

```rust
PackageJsonGenerator
TsConfigGenerator
MoonMetadataGenerator
```

## Generator Interface

```rust
pub trait Generator {
    fn generate(&self);
    fn validate(&self);
}
```

## Validation Service

Responsible for consistency checks.

### Responsibilities

```text
Workspace Validation
Configuration Validation
Generated File Validation
Toolchain Validation
```

### Example

```sh
luna doctor
```

Uses:

```rust
ValidatorRegistry
```

## Validator Interface

```rust
pub trait Validator {
    fn validate(&self);
}
```

## Adapter Service

Adapters provide ecosystem integration.

### Responsibilities

```text
Dependency Installation
Dependency Updates
Dependency Discovery
Package Metadata
```

### Examples

```text
BunAdapter
UvAdapter
GoAdapter
CargoAdapter
```

## Adapter Registry

```rust
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn Adapter>>,
}
```

### Lookup

```rust
registry.adapter("typescript")
registry.adapter("python")
```

## Execution Service

Responsible for external processes.

### Responsibilities

```text
Process Execution
Environment Setup
Output Streaming
Error Handling
```

### Examples

```sh
moon run
proto install
bun add
uv sync
```

## Process Runner

Single implementation.

### Example

```rust
runner.execute("moon", &["run", "build"]);
```

### Responsibilities

```text
stdout
stderr
exit codes
cwd
environment
```

## Infrastructure Layer

Infrastructure should remain isolated.

### Responsibilities

```text
Filesystem
Processes
Serialization
Template Rendering
```

### Modules

```text
filesystem.rs
process.rs
yaml.rs
json.rs
templates.rs
```

## Command Architecture

Commands should be thin.

### Example

```rust
pub struct SyncCommand;
```

Responsibilities:

```text
Load Runtime
Call Services
Display Results
```

### Non-Responsibilities

```text
Generation Logic
Discovery Logic
Validation Logic
```

## Command Modules

```text
commands/
├── sync.rs
├── doctor.rs
├── build.rs
├── projects.rs
├── graph.rs
├── add.rs
├── update.rs
└── create.rs
```

## Service Communication

Commands never communicate directly.

### Correct

```text
Command
  ↓
Runtime
  ↓
Service
```

### Incorrect

```text
Command
  ↓
Generator
  ↓
Filesystem
```

## Future Daemon

Not implemented in Phase 1.

### Motivation

Large workspaces eventually benefit from:

```text
Cached Discovery
Cached Graphs
File Watching
Background Validation
```

### Future Command

```sh
luna daemon
```

### Architecture

```text
CLI
 │
 ▼
IPC
 │
 ▼
Daemon
 │
 ▼
Runtime
```

## Future Plugin System

Not implemented initially.

### Goal

Allow external extensions.

Example:

```text
Terraform Adapter
Docker Adapter
Kubernetes Adapter
```

### Future Structure

```text
plugins/
```

or

```text
~/.luna/plugins
```

## Error Handling

All runtime errors should be typed.

### Example

```rust
enum LunaError {
    WorkspaceNotFound,
    ProjectNotFound,
    InvalidConfiguration,
}
```

## Logging

Use structured logging.

Recommended:

```text
tracing
```

### Levels

```text
error
warn
info
debug
trace
```

## Telemetry

Out of scope.

Telemetry is out of scope for the current refactor.

## Testing Strategy

### Unit Tests

Test:

```text
Workspace
Discovery
Generators
Validators
```

### Integration Tests

Test:

```text
moon integration
proto integration
adapter execution
```

### Snapshot Tests

Test:

```text
Generated package.json
Generated tsconfig.json
Generated metadata
```

## Acceptance Criteria

The runtime architecture is complete when:

- Service boundaries are defined
- Runtime owns orchestration
- Commands remain thin
- Infrastructure remains isolated
- Future daemon architecture is supported
- Future plugin architecture is possible
- Moon integration remains clean
- Proto integration remains clean

## Architectural Decision

### Decision

Use a service-oriented runtime architecture with strict ownership boundaries.

### Rationale

This minimizes coupling and allows Luna to evolve without becoming a monolithic tool.

### Consequences

#### Positive

- Testable
- Modular
- Extensible
- Maintainable

#### Tradeoffs

- More initial structure
- More abstraction layers
- Additional runtime complexity
