---
title: "RFC-0004: Luna Refactor Roadmap"
description: "Implementation roadmap with epics, stories, tasks, acceptance criteria, and migration sequencing sourced from the architecture RFCs."
status: draft
rfc: "0004"
type: rfc
depends_on:
  - RFC-0001
  - RFC-0002
  - RFC-0003
  - RFC-0005
  - RFC-0006
  - RFC-0007
  - RFC-0008
referenced_by: []
---

## Overview

This RFC defines the implementation roadmap for the Luna refactor.

The roadmap is organized into:

- Epics
- Stories
- Tasks
- Acceptance Criteria

The goal is to deliver Luna incrementally without disrupting existing projects.

The migration strategy prioritizes:

1. Stability
2. Backward compatibility
3. Incremental adoption
4. Clear ownership boundaries

## Guiding Principles

### Principle 1

Do not replace Moon.

Moon already solves:

- Task orchestration
- Dependency graphs
- Caching
- Affected detection

Luna should integrate with Moon.

### Principle 2

Do not replace Proto.

Proto already solves:

- Toolchain management
- Version pinning
- Runtime discovery

Luna should integrate with Proto.

### Principle 3

Do not replace package managers.

Luna delegates to:

```text
Bun
uv
Go
Cargo
```

Luna does not resolve dependency graphs.

### Principle 4

Minimize disruption.

Existing applications should continue functioning throughout the migration.

## Roadmap Overview

```text
Phase 1
├─ Workspace Runtime Foundation

Phase 2
├─ Workspace Registry

Phase 3
├─ Configuration Generation

Phase 4
├─ Workspace Intelligence

Phase 5
├─ Dependency Orchestration

Phase 6
├─ Luna Native Workspace
```

## Phase 1

## Workspace Runtime Foundation

### Objective

Introduce Luna as a Rust-based runtime.

No project behavior changes.

No generation.

No migration.

## Epic 1

Rust CLI Foundation

### Story 1.1

Create Luna CLI Workspace

#### Tasks

- Create `crates/luna-cli`
- Configure Cargo workspace
- Configure release builds
- Configure linting
- Configure formatting

#### Acceptance Criteria

- CLI compiles
- Binary runs
- CI succeeds

### Story 1.2

Command Router

#### Tasks

Implement:

```sh
luna
luna help
luna version
```

#### Acceptance Criteria

- Commands route correctly
- Help output exists

### Story 1.3

Workspace Root Discovery

#### Tasks

Detect:

```text
luna.yml
.prototools
```

#### Acceptance Criteria

- Workspace root discovered
- Errors are descriptive

### Story 1.4

Process Execution Layer

#### Tasks

Create:

```rust
ProcessRunner
```

Responsibilities:

- stdout
- stderr
- exit codes
- working directory

#### Acceptance Criteria

- Commands execute reliably

## Epic 2

Workspace Doctor

### Story 2.1

Toolchain Validation

#### Tasks

Validate:

```text
proto
moon
bun
uv
go
cargo
```

#### Acceptance Criteria

- Missing tools detected
- Versions reported

### Story 2.2

Workspace Validation

#### Tasks

Validate:

```text
luna.yml
.prototools
moon workspace
```

#### Acceptance Criteria

- Invalid workspace detected

### Story 2.3

Project Validation

#### Tasks

Verify:

```text
Registered Projects
Existing Projects
Missing Projects
```

#### Acceptance Criteria

- Drift reported

## Phase 2

## Workspace Registry

### Objective

Introduce Luna metadata ownership.

## Epic 3

Workspace Model

### Story 3.1

luna.yml Schema

#### Tasks

Define:

```yaml
workspace:
projects:
toolchains:
```

#### Acceptance Criteria

- Schema documented
- Validation implemented

### Story 3.2

Project Registry

#### Tasks

Implement:

```yaml
projects:
```

#### Acceptance Criteria

- Projects load correctly

### Story 3.3

Project Discovery

#### Tasks

Discover via:

1. luna.yml
2. moon
3. filesystem

#### Acceptance Criteria

- Discovery is deterministic

### Story 3.4

Workspace API

#### Tasks

Implement:

```rust
Workspace
WorkspaceProject
WorkspaceConfig
```

#### Acceptance Criteria

- Shared API exists

## Phase 3

## Configuration Generation

### Objective

Reduce configuration sprawl.

## Epic 4

Sync Engine

### Story 4.1

Sync Command

#### Tasks

Implement:

```sh
luna sync
```

#### Acceptance Criteria

- Command executes
- Dry run supported

### Story 4.2

Drift Detection

#### Tasks

Compare:

```text
Expected
Actual
```

#### Acceptance Criteria

- Drift reported
- Drift repaired

### Story 4.3

Generated File Tracking

#### Tasks

Track:

```text
Version
Hash
Generator
```

#### Acceptance Criteria

- Metadata stored

## Epic 5

TypeScript Generation

### Story 5.1

Package Manifest Generation

#### Tasks

Generate:

```text
package.json
```

#### Acceptance Criteria

- Generated correctly
- Stable output

### Story 5.2

TypeScript Configuration Generation

#### Tasks

Generate:

```text
tsconfig.json
```

#### Acceptance Criteria

- Valid TS config

### Story 5.3

Workspace References

#### Tasks

Generate:

```text
Project references
```

#### Acceptance Criteria

- References remain synchronized

## Phase 4

## Workspace Intelligence

### Objective

Expose workspace knowledge.

## Epic 6

Workspace Discovery

### Story 6.1

Projects Command

#### Tasks

Implement:

```sh
luna projects
```

#### Acceptance Criteria

- Lists projects

### Story 6.2

JSON Output

#### Tasks

Implement:

```sh
luna projects --json
```

#### Acceptance Criteria

- Machine-readable output

## Epic 7

Workspace Graph

### Story 7.1

Graph Command

#### Tasks

Implement:

```sh
luna graph
```

#### Acceptance Criteria

- Displays project graph

### Story 7.2

Moon Graph Integration

#### Tasks

Consume:

```sh
moon query
```

#### Acceptance Criteria

- Graph reflects Moon

### Story 7.3

Dependency Visualization

#### Tasks

Display:

```text
Projects
Dependencies
Layers
```

#### Acceptance Criteria

- Visualization works

## Phase 5

## Dependency Orchestration

### Objective

Create unified dependency workflows.

## Epic 8

Adapter System

### Story 8.1

Adapter Registry

#### Tasks

Implement:

```rust
AdapterRegistry
```

#### Acceptance Criteria

- Adapters register dynamically

### Story 8.2

Bun Adapter

#### Tasks

Support:

```sh
bun add
bun update
bun outdated
```

#### Acceptance Criteria

- Commands delegated correctly

### Story 8.3

uv Adapter

#### Tasks

Support:

```sh
uv add
uv sync
uv lock
```

#### Acceptance Criteria

- Commands delegated correctly

### Story 8.4

Go Adapter

#### Tasks

Support:

```sh
go get
go mod tidy
```

#### Acceptance Criteria

- Commands delegated correctly

### Story 8.5

Cargo Adapter

#### Tasks

Support:

```sh
cargo add
cargo update
```

#### Acceptance Criteria

- Commands delegated correctly

## Epic 9

Dependency Commands

### Story 9.1

Add Command

#### Tasks

Implement:

```sh
luna add
```

#### Acceptance Criteria

- Routes to adapter

### Story 9.2

Remove Command

#### Tasks

Implement:

```sh
luna remove
```

#### Acceptance Criteria

- Routes to adapter

### Story 9.3

Update Command

#### Tasks

Implement:

```sh
luna update
```

#### Acceptance Criteria

- Supports all adapters

### Story 9.4

Outdated Command

#### Tasks

Implement:

```sh
luna outdated
```

#### Acceptance Criteria

- Aggregates adapter results

## Phase 6

## Luna Native Workspace

### Objective

Allow Luna to become the primary workspace interface.

## Epic 10

Project Creation

### Story 10.1

Create Application

#### Tasks

Implement:

```sh
luna create app
```

#### Acceptance Criteria

- Project scaffolded

### Story 10.2

Create Library

#### Tasks

Implement:

```sh
luna create library
```

#### Acceptance Criteria

- Library scaffolded

### Story 10.3

Template Registry

#### Tasks

Support:

```text
TypeScript
Python
Go
Rust
```

#### Acceptance Criteria

- Templates discoverable

## Epic 11

Workspace Evolution

### Story 11.1

Generated Moon Metadata

#### Tasks

Generate:

```text
Project metadata
Tags
Layers
```

#### Acceptance Criteria

- Moon metadata synchronized

### Story 11.2

Future Python Generation

#### Tasks

Evaluate:

```text
pyproject.toml
```

#### Acceptance Criteria

- Decision documented

### Story 11.3

Future Go Generation

#### Tasks

Evaluate:

```text
go.mod
```

#### Acceptance Criteria

- Decision documented

### Story 11.4

Future Cargo Generation

#### Tasks

Evaluate:

```text
Cargo.toml
```

#### Acceptance Criteria

- Decision documented

## Migration Strategy

### Phase 1

Current:

```sh
bun run dev
```

Supported.

### Phase 2

Preferred:

```sh
luna dev
```

Supported.

### Phase 3

Generated configs introduced.

```sh
luna sync
```

Required.

### Phase 4

Workspace registry becomes authoritative.

### Phase 5

Legacy root scripts deprecated.

## Success Criteria

The roadmap is successful when:

- Luna becomes primary developer interface
- Moon remains execution source of truth
- Proto remains toolchain source of truth
- Configuration duplication is reduced
- New projects require minimal setup
- Additional ecosystems can be added without redesign

## Architectural Decision

### Decision

Implement Luna incrementally through six phases.

### Rationale

This minimizes risk while allowing Luna to evolve into a first-class workspace runtime.

### Consequences

#### Positive

- Low migration risk
- Backward compatibility
- Clear ownership boundaries
- Gradual adoption

#### Tradeoffs

- Longer implementation timeline
- Temporary duplication
- Multiple transition states
