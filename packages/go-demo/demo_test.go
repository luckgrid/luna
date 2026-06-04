package demo

import "testing"

func TestGreet(t *testing.T) {
	if got := Greet(""); got != "hello, luna" {
		t.Fatalf("Greet(\"\") = %q, want hello, luna", got)
	}
}
