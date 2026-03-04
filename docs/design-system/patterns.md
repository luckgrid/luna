# Patterns

Patterns are grouped by UI dimensions and authored as classless-first CSS.

## Dimensions

- `layout`
- `action`
- `navigation`
- `form`
- `display`
- `feedback`

## Current Pattern Surface

- **Layout**: page/container flow plus scoped context rules for nested sections.
- **Action**: button variants and dialog patterns.
- **Navigation**: links, nav, breadcrumb-like structures.
- **Form**: classless form controls and states.
- **Display**: tables, code/figure/misc primitives, accordion via `details`.
- **Feedback**: tooltip, progress, loading states.

## Utilities

- `stack`: vertical rhythm helper.
- `cluster`: wrapping inline group helper.
- `panel`: tokenized container shell.

## Authoring Rules

- Prefer element selectors first; use utility classes only for explicit overrides.
- Keep utilities low-level and semantic (no broad atomic matrix).
- Use token variables exclusively for color, spacing, stroke, radius, and typography.
- Keep DS generic; application composition belongs in `@luna/ui`.
