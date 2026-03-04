# Tokens

Tokens are split into two layers:

- `src/tokens/properties.css`: low-level, theme-agnostic properties
- `src/tokens/theme.css`: semantic and contextual design tokens

## Properties (Low-Level)

- Color properties:
  - monochrome scale (`--color-black` ... `--color-white`)
  - ROYGBIV-like scales (`--color-dark-red-0` ... `--color-light-violet-0`)
- Sizing properties:
  - `--screen-min`, `--screen-max`
  - `--space-min`, `--space-max`
  - `--radius-min`, `--radius-max`
  - `--stroke-min`, `--stroke-max`
  - `--text-min`, `--text-max`
  - `--text-ratio`
  - `--stroke-offset`
- Multipliers:
  - `--multiplier-xs`, `--multiplier-sm`, `--multiplier-md`

## Theme (Semantic)

- Semantic colors (`--color-bg`, `--color-fg`, `--color-border`, `--color-ring`, plus neutral/primary/secondary/accent/muted/alert groups).
- Fluid scales:
  - `--space-0..4`
  - `--radius-0..4`
  - `--stroke-0..4`
- Screen breakpoints: `--screen-0..11`
- Typography:
  - weight scale (`--font-weight-*`)
  - line-height semantics (`--line-height-none`, `--line-height-tight`, `--line-height-snug`, `--line-height-normal`, `--line-height-relaxed`, `--line-height-loose`)
  - size scale (`--text-xs` ... `--text-3xl`)
  - companion tokens per size (`--text-<size>--line-height`, `--text-<size>--letter-spacing`, `--text-<size>--font-weight`)

## Rules

- Keep raw values in properties; keep semantic decisions in theme.
- Reuse fluid ratio and multipliers instead of duplicating formulas.
- Add new tokens only when a real DS pattern needs them.
