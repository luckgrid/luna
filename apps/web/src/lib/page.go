package lib

import (
	"context"
	"os"
	"path/filepath"

	"github.com/a-h/templ"
)

// WritePage renders a templ component to dist/<rel>, creating parent
// directories as needed.
func WritePage(distDir, rel string, c templ.Component) error {
	out := filepath.Join(distDir, rel)
	if err := os.MkdirAll(filepath.Dir(out), 0o755); err != nil {
		return err
	}
	f, err := os.Create(out)
	if err != nil {
		return err
	}
	defer f.Close()
	return c.Render(context.Background(), f)
}
