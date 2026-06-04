# go-demo

Demo [`go.work`](../../go.work) workspace member — not for production.

Proves root `go work sync` (via `luna install`) resolves local modules the same way `uv` / `cargo` workspaces do. [`apps/web`](../web/) imports this package in `workspace/link.go` so the Hugo module stays in the same workspace graph.

```sh
moon run go-demo:test
go test ./packages/go-demo/...
```
