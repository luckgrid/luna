// Package workspace keeps apps/web in the go.work graph with packages/go-demo.
// Hugo itself is a go tool dependency; this file is the workspace library link.
package workspace

import "github.com/luckgrid/luna/packages/go-demo"

// Greet forwards to the shared demo module (workspace-local, not from the module proxy).
func Greet(name string) string {
	return demo.Greet(name)
}
