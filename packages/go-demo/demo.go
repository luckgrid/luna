// Package demo is a placeholder Go workspace member (not for production use).
package demo

// Greet returns a demo greeting for workspace wiring checks.
func Greet(name string) string {
	if name == "" {
		name = "luna"
	}
	return "hello, " + name
}
