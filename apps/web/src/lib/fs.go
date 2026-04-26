package lib

import (
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
)

// CopyTree copies src into dst recursively. If src doesn't exist it is a no-op.
func CopyTree(src, dst string) error {
	if _, err := os.Stat(src); os.IsNotExist(err) {
		return nil
	}
	return filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		rel, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}
		target := filepath.Join(dst, rel)
		if info.IsDir() {
			return os.MkdirAll(target, 0o755)
		}
		return copyFile(path, target)
	})
}

func copyFile(src, dst string) error {
	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return err
	}
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()
	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()
	_, err = io.Copy(out, in)
	return err
}

// ServeDir starts a tiny static file server for local preview. Blocks until
// the process is terminated.
func ServeDir(distDir, port string) error {
	addr := ":" + port
	log.Printf("serving %s on http://localhost%s", distDir, addr)
	mux := http.NewServeMux()
	mux.Handle("/", http.FileServer(http.Dir(distDir)))
	return http.ListenAndServe(addr, mux)
}
