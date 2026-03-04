# Minimal CSS Design System Overview

Luna's design system is CSS-first, framework-agnostic, and intentionally lean.

## Principles

- Keep low-level tokens in `properties.css` and semantic/theme tokens in `theme.css`.
- Prefer classless element styling for baseline UX, then layer opt-in utilities.
- Use modern CSS features (`@layer`, `@scope`, `clamp()`, `color-mix()`, `light-dark()`).
- Keep product-specific composition in `@luna/ui`, not inside DS core.

## Layer Contract

```css
@layer tokens, base, layout, action, navigation, form, display, feedback, utilities, overrides;
```

## Current Scope

- Token foundations: monochrome + spectrum color properties, scale multipliers, screen bounds, min/max size properties.
- Semantic theme tokens: contextual colors, fluid space/radius/stroke, and typography scales.
- Classless UI layers: layout, action, navigation, form, display, feedback.
- Utilities: low-level composition helpers (`stack`, `cluster`, `panel`).

## Consumption

Import DS CSS directly:

```css
@import "@luna/ds/styles.css";
```

For Vite projects, DS can also provide package-level Lightning CSS defaults via DS Vite integration.
