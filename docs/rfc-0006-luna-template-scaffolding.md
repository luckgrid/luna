---
title: "RFC-0006: Luna Template and Scaffolding System"
description: "Specification for project creation, scaffolding, the template registry, workspace registration, and template standards."
status: draft
rfc: "0006"
type: rfc
depends_on:
  - RFC-0001
  - RFC-0002
  - RFC-0003
  - RFC-0005
referenced_by:
  - RFC-0008
---

## Overview

This RFC defines the Luna template and scaffolding system.

The template system is responsible for:

- Creating new projects
- Creating new packages
- Creating new applications
- Creating new services
- Generating initial workspace metadata
- Generating standard configuration
- Applying organization conventions

The goal is to eliminate repetitive setup while preserving developer control.

## Problem Statement

Today, creating a new project often requires:

```text
Create Directory
Create package.json
Create tsconfig.json
Create moon.yml
Register Project
Create README
Create Build Scripts
Configure CI
Configure Linting
```

This process becomes increasingly expensive as:

- Languages increase
- Frameworks increase
- Standards evolve
- Team size grows

## Goals

### Goal 1

Standardize project creation.

### Goal 2

Reduce onboarding time.

### Goal 3

Ensure generated projects conform to workspace standards.

### Goal 4

Support multiple ecosystems.

Examples:

```text
TypeScript
Python
Go
Rust
```

### Goal 5

Allow future customization.

## Non-Goals

### Not a Framework Generator

Luna should not attempt to replace:

```text
create-vite
create-next-app
cargo new
uv init
hugo new
```

Instead, Luna should orchestrate them when appropriate.

### Not a Code Generator

Luna is generating:

```text
Project Structure
Workspace Metadata
Configuration
```

not application logic.

## Design Principles

### Convention Over Configuration

Generated projects should follow Luna conventions by default.

### Explicit Metadata

Every generated project must register itself with:

```yaml
projects:
```

inside:

```text
luna.yml
```

### Reproducibility

Templates must be identifiable and reproducible.

A project generated today should be reproducible tomorrow without requiring a release-managed migration system.

## High-Level Architecture

```text
Create Command
        │
        ▼

Template Registry
        │
        ▼

Template Renderer
        │
        ▼

Project Generator
        │
        ▼

Workspace Registration
```

## Commands

### Create Application

```sh
luna create app my-app
```

### Create Library

```sh
luna create library ui
```

### Create Service

```sh
luna create service api
```

### Create Package

```sh
luna create package shared
```

## Future Commands

### Interactive Mode

```sh
luna create
```

Example:

```text
What would you like to create?

> Application
> Library
> Service
> Package
```

## Template Registry

Templates are discovered through a registry.

### Initial Structure

```text
templates/
├── applications/
├── libraries/
├── services/
└── packages/
```

## Language Templates

Each category supports multiple languages.

Example:

```text
templates/
└── applications/
    ├── typescript/
    ├── python/
    ├── go/
    └── rust/
```

## Framework Templates

Optional framework layer.

Example:

```text
typescript/
├── solid-start/
├── react/
├── node/
└── vite/
```

## Template Metadata

Each template contains metadata.

Example:

```yaml
name: solid-start
language: typescript
type: application
template: solid-app
```

## Template Variables

Templates support variables.

Example:

```yaml
project:
  name: ui
  language: typescript
```

## Available Variables

### Project Name

```text
{{ project.name }}
```

### Language

```text
{{ project.language }}
```

### Framework

```text
{{ project.framework }}
```

### Workspace Name

```text
{{ workspace.name }}
```

## Template Rendering

Recommended Engines:

```text
Minijinja
Tera
Handlebars
```

Avoid:

```text
Manual String Replacement
```

## Generated Structure

Example:

```sh
luna create library ui
```

Produces:

```text
packages/
└── ui/
    ├── src/
    ├── README.md
    ├── package.json
    └── tsconfig.json
```

## Workspace Registration

Creation automatically updates:

```text
luna.yml
```

Example:

```yaml
projects:
  ui:
    language: typescript
    type: library
```

## Project Types

### Application

Deployable software.

Examples:

```text
SolidStart
FastAPI
Hugo
```

### Library

Reusable code.

Examples:

```text
UI
Utilities
SDKs
```

### Service

Backend services.

Examples:

```text
API
Workers
Jobs
```

### Tooling

Workspace tooling.

Examples:

```text
CLI
Generators
Scripts
```

## Template Identity

Every template must have a stable identifier.

### Example

```yaml
template: solid-app
language: typescript
type: application
```

## Why Identity Matters

Without stable template identity:

```text
Generated Projects Become Hard To Trace
```

## Template Changes

Template changes should be handled as direct edits while Luna remains in early pre-alpha development.

Responsibilities:

```text
Review Template Metadata
Preview Generated Files
Apply Changes Intentionally
```

## Built-In Templates

Luna ships with:

```text
TypeScript Library
TypeScript Application
Python Service
Go Service
Rust Service
```

## Community Templates

Future support.

Example:

```sh
luna template install
```

Not part of the current refactor.

## Template Composition

Templates should support inheritance.

### Example

```text
Base Library
        │
        ▼
TypeScript Library
        │
        ▼
React Library
```

## Template Structure

```text
template/
├── template.yml
├── files/
├── hooks/
└── generators/
```

## Hooks

Templates may define hooks.

### Before Create

```text
Validate Environment
```

### After Create

```text
Install Dependencies
Register Workspace
```

## Workspace Standards

Templates should automatically apply:

```text
Linting
Formatting
Documentation
Metadata
```

## CI Integration

Future templates may generate:

```text
CI Configurations
Release Workflows
Deployment Workflows
```

Not required for the current refactor.

## Validation

Templates must validate:

```text
Required Variables
Language Support
Framework Support
```

## Error Handling

Examples:

```text
Template Not Found
Framework Not Supported
Invalid Variable
```

## Acceptance Criteria

The template system is complete when:

- New projects can be scaffolded
- Workspace registration is automatic
- Templates have stable identities
- Templates are language aware
- Templates support variables
- Templates are reproducible
- Future community templates are possible

## Architectural Decision

### Decision

Use an identifiable template registry with composable templates and automatic workspace registration.

### Rationale

Templates become a strategic capability that enables Luna to standardize workspace creation without becoming a framework generator.

### Consequences

#### Positive

- Consistent project creation
- Reduced onboarding time
- Reproducible project setup
- Easier workspace maintenance

#### Tradeoffs

- Template maintenance burden
- Template change-management complexity
- Additional abstraction layer
