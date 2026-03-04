# Roadmap

## Current State

- CSS-first DS in place with layered architecture.
- Properties/theme token split implemented.
- Classless pattern layers shipped for layout/action/navigation/form/display/feedback.
- Utility surface intentionally minimal (`stack`, `cluster`, `panel`).

## Next: Token Hardening

- Audit token naming consistency and remove remaining legacy aliases.
- Expand semantic color docs with usage intent and contrast guidance.
- Add token change policy/versioning notes.

## Next: Pattern Expansion

- Add additional layout primitives (`grid`, `split`, `center`) only when needed.
- Add optional composed patterns (cards, tabs, badges) built on existing tokens.
- Keep new patterns classless-first and dimension-scoped.

## Next: DX and Validation

- Add lightweight visual token/pattern preview docs.
- Add regression checks for DS CSS build output.
- Add accessibility checks for contrast and focus-visible across themes.
