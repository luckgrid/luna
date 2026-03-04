# Functions and Modern CSS Patterns

The DS uses native CSS features over JS config.

## Required Patterns

- `clamp()` for fluid token interpolation (`space`, `radius`, `stroke`, text companions).
- `calc()` for token composition and ratio-based interpolation.
- `color-mix()` for rings, overlays, and subtle semantic derivations.
- `light-dark()` for semantic dual-theme color tokens.
- `@layer` to guarantee override strategy.
- `@scope` for contextual nested layout/component styling.

## Current Fluid Formulas

Typography uses a dedicated ratio token:

```css
--text-ratio: calc((100vw - var(--screen-min)) / (var(--screen-max) - var(--screen-min)));
```

This ratio is reused by text size and typography companion tokens.

Space uses its own fluid interpolation (and radius/stroke follow the same pattern with their own min/max properties):

```css
--space-0: clamp(
  var(--space-min),
  calc(
    var(--space-min) + (var(--space-max) - var(--space-min)) *
      ((100vw - var(--screen-min)) / (var(--screen-max) - var(--screen-min)))
  ),
  var(--space-max)
);
```

## Guidance

- Keep math in tokens and consume tokens in patterns/components.
- Prefer property tokens (`properties.css`) for raw values and `theme.css` for semantic aliases.
- Avoid preprocess-only syntax not supported by the current toolchain.
