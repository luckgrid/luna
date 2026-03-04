# Architecture

## Build and Runtime

- Author CSS in `packages/ds`.
- Consume via Vite in apps.
- Compile/minify with Lightning CSS in the app pipeline.

## Source Layout

```text
packages/ds/
  styles.css
  src/
    tokens/
      properties.css
      theme.css
    base.css
    ui/
      layout.css
      action.css
      navigation.css
      form.css
      display.css
      feedback.css
    utilities.css
```

## Entrypoint Contract

`styles.css` is the single DS entrypoint and owns:

- `@layer` declaration order
- import order for all DS modules

Layer order:

```css
@layer tokens, base, layout, action, navigation, form, display, feedback, utilities, overrides;
```

Apps should import only DS entry CSS, then place app overrides after DS.

## Styling Model

- `properties.css`: low-level primitives (`--color-*`, min/max properties, multipliers, `--text-ratio`, `--stroke-offset`).
- `theme.css`: semantic/contextual tokens (`--color-*` semantic aliases, fluid scales, text companions).
- `ui/*`: classless patterns grouped by UI dimension.
- `utilities.css`: low-level composition helpers, not a large utility matrix.
