# `py-demo`

Demo Python workspace member — reference layout for new Python packages and apps in Luna.

**Not for production use.** This package exists to show how internal Python code is structured with scoped per-member config while the uv workspace, lockfile, and `.venv` stay at the repo root.

## Why this layout?

Python members use two patterns depending on whether they are **apps** or **libraries**:

| Kind                             | Path                                  | Installed?                       | Import / run                                                                |
| -------------------------------- | ------------------------------------- | -------------------------------- | --------------------------------------------------------------------------- |
| **App** (`apps/api`)             | `apps/<app>/src/*`                    | No (`[tool.uv] package = false`) | `PYTHONPATH=src`, bare imports (`from config import …`, `uvicorn main:app`) |
| **Library** (`packages/py-demo`) | `packages/<lib>/src/<import_name>.py` | Yes (hatchling editable)         | `from py_demo import greet`                                                 |

The project directory name is the namespace; `src/` holds source without repeating it.

### Library layout (`py-demo`)

```text
packages/py-demo/
├── pyproject.toml      # identity, build, hatch sources = ["src"]
├── moon.yml            # language + layer (inherits shared tasks)
├── src/
│   └── py_demo.py      # importable module (distribution import name)
└── tests/
    └── test_smoke.py
```

Build config (editable-safe prefix strip):

```toml
[tool.hatch.build.targets.wheel]
only-include = ["src"]
sources = ["src"]
```

Tests and consumers import by **module name**:

```python
from py_demo import greet
```

### App layout (`apps/api`)

```text
apps/api/
├── pyproject.toml      # identity + deps; [tool.uv] package = false
├── moon.yml
├── src/
│   ├── main.py         # FastAPI entrypoint
│   ├── config.py
│   └── ai/             # subpackages as needed
└── tests/
```

Apps are **not** installed into the shared `.venv`, so generic top-level module names (`main`, `config`) do not collide with other members. Moon sets `PYTHONPATH=src` for dev/start; pytest uses `pythonpath = ["src"]` in the app's `pyproject.toml`.

See [apps/api/README.md](../../apps/api/README.md) for the full app example.

### Scoped per-member config

Shared tooling lives at the root; each member owns run/test/build config:

| What                       | Where                                                                                                                                  |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Ruff rules                 | root [`pyproject.toml`](../../pyproject.toml) `[tool.ruff]`                                                                            |
| Dev deps (ruff, pytest, …) | root `[dependency-groups].dev`                                                                                                         |
| Workspace lockfile + venv  | root [`uv.lock`](../../uv.lock) + [`.venv`](../../.venv)                                                                               |
| Lint/format/test tasks     | per-member moon ([`.moon/tasks/py-lib.yml`](../../.moon/tasks/py-lib.yml) or [`.moon/tasks/py-api.yml`](../../.moon/tasks/py-api.yml)) |
| Pytest (apps)              | member `pyproject.toml` `[tool.pytest.ini_options]`                                                                                    |
| Build (libs)               | member `pyproject.toml` `[build-system]` + hatch                                                                                       |

Run lint/format per member:

```sh
moon run py-demo:lint
moon run api:lint
```

## Workspace dependency

`apps/api` depends on this package via uv workspace sources:

```toml
# apps/api/pyproject.toml
dependencies = ["py-demo"]

[tool.uv.sources]
py-demo = { workspace = true }
```

After `luna install` (`uv sync`), changes here are picked up by `api` without reinstalling.

## Commands

From the workspace root:

```sh
moon run py-demo:test
luna test py-demo    # if using luna test with project filter
```

From this directory (after `uv sync` at root):

```sh
uv run pytest tests/
```

## Add a new Python package or app

**Library** (`packages/<name>/`):

1. Create `src/<import_name>.py` (or `src/<import_name>/` for multi-module libs).
2. Add `pyproject.toml` with `[build-system]`, hatch `only-include = ["src"]` + `sources = ["src"]`.
3. Register in root [`pyproject.toml`](../../pyproject.toml) `[tool.uv.workspace].members`.
4. Add `moon.yml` with `language: python`, `layer: library`.

**App** (`apps/<name>/`):

1. Create `src/main.py` and modules under `src/`.
2. Add `pyproject.toml` with `[tool.uv] package = false`, scoped `[tool.pytest.ini_options]` (`pythonpath = ["src"]`).
3. Register in root workspace members.
4. Add `moon.yml` with `language: python`, `layer: application`.

For inter-member deps, add `[tool.uv.sources] name = { workspace = true }` in the consumer. Run `luna install`.

See the root [README Configuration map](../../README.md#configuration-map) for the full Python workspace overview.
