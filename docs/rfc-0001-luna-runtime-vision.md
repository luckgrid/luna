---
title: "RFC-0001: Luna Runtime Vision"
description: "Defines why Luna exists, the workspace runtime philosophy, ownership boundaries, high-level architecture, and design principles."
status: draft
rfc: "0001"
type: rfc
depends_on:
  - RFC-0000
referenced_by:
  - RFC-0002
  - RFC-0003
  - RFC-0005
  - RFC-0006
  - RFC-0007
  - RFC-0008
---

## Executive Summary

Luna currently provides a strong polyglot monorepo foundation built on:

- Proto for toolchain management
- Moon for orchestration and project graph management
- Bun workspaces for JavaScript and TypeScript projects
- uv for Python projects
- Go modules for Hugo and Go projects

However, the current developer experience remains Bun-centric at the workspace root.

Examples:

```sh
bun run setup
bun run dev
bun run build
bun run check
```

This creates the perception that Luna is a JavaScript monorepo that happens to contain Python and Go projects.

The objective of this refactor is to reposition Luna as a true polyglot workspace runtime where:

- Proto manages toolchains
- Moon manages execution
- Luna manages workspace definition and developer experience
- Language ecosystems remain native and independent

After completion, the preferred developer workflow becomes:

```sh
luna setup
luna sync
luna dev
luna build
luna check
luna update
```

The repository should feel language-neutral.

TypeScript, Python, Go, Rust, and future ecosystems become first-class citizens.

## Architectural Goals

### Goal 1

Remove TypeScript/Bun ownership of the workspace.

Bun should become:

```text
JavaScript / TypeScript Adapter
```

rather than:

```text
Workspace Runtime
```

### Goal 2

Establish Luna as the Workspace Runtime.

Luna becomes responsible for:

- Workspace definition
- Workspace synchronization
- Project discovery
- Configuration generation
- Dependency orchestration
- Developer experience

Luna does **not** become:

- Package manager
- Build system
- Dependency resolver
- Task runner

### Goal 3

Reduce Configuration Sprawl.

Current repositories often contain:

```text
package.json
tsconfig.json
vite.config.ts
eslint.config.js
moon.yml
moon tasks
```

for every project.

This leads to:

- Duplicated metadata
- Configuration drift
- Maintenance overhead
- Onboarding complexity

The long-term goal is:

```text
Human Authored
    ↓
luna.yml

Generated
    ↓
moon config
package.json
tsconfig.json
workspace manifests

Application Owned
    ↓
framework-specific config
```

## Vision

Luna should become a workspace runtime.

Not:

```text
Another package manager
```

Not:

```text
Another build system
```

But instead:

```text
Workspace Runtime
+
Workspace Registry
+
Config Generator
+
Developer Experience Layer
```

## Runtime Layer Model

```text
Luna
│
├─ Workspace Runtime
├─ Config Generation
├─ Project Discovery
├─ Developer Experience
│
├───────────────┐
│               │
▼               ▼

Moon         Proto

Execution    Toolchains

│               │
│               │
▼               ▼

Adapters

├─ Bun
├─ uv
├─ Go
├─ Cargo
└─ Future Toolchains
```

## Guiding Principles

### Luna Owns

- Workspace metadata
- Workspace discovery
- Configuration generation
- Developer commands
- Project registration

### Moon Owns

- Execution
- Task graph
- Dependency graph
- Caching
- Affected detection

### Proto Owns

- Toolchain installation
- Toolchain versioning
- Runtime discovery

### Adapters Own

- Dependency resolution
- Dependency installation
- Ecosystem-native operations

Luna never replaces ecosystem-native tooling.
