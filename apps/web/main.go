// Package main is the entry point for the Luna static site generator.
//
// Keep this file lean: flags + template wiring + a small SSG runner.
//
// `src/pages/` contains templ-only page modules, while `src/content/`
// holds the Markdown content the SSG renders.
//
//   - src/pages/index.templ            → /index.html  (via pages.Index)
//   - src/content/<name>.md            → /<name>.html
//     rendered via a dedicated template (registered in Site.Pages)
//   - src/content/<catalog>/*.md       → /<catalog>/<slug>/ (catalog items)
//     rendered via the catalog templates registered in main.go
//
// With `--serve`, after generation it starts a tiny static file server
// for local preview.
package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"sort"

	"github.com/a-h/templ"
	"github.com/luckgrid/luna/apps/web/src/lib"
	"github.com/luckgrid/luna/apps/web/src/pages"
	"github.com/luckgrid/luna/apps/web/src/pages/posts"
)

// --- SSG types ---

type Catalog struct {
	Name   string
	Index  func(posts []lib.Post) templ.Component
	Detail func(p lib.Post) templ.Component
}

type Site struct {
	Root     string
	Index    func(latest []lib.Post) templ.Component
	Pages    map[string]func(p lib.Post) templ.Component
	Catalogs []Catalog
}

// --- SSG runner ---

func Build(s Site) error {
	contentDir := filepath.Join(s.Root, "src", "content")
	publicDir := filepath.Join(s.Root, "public")
	distDir := filepath.Join(s.Root, "dist")

	if err := os.MkdirAll(distDir, 0o755); err != nil {
		return err
	}

	var latest []lib.Post
	for _, c := range s.Catalogs {
		posts, err := lib.ParseDir(c.Name, filepath.Join(contentDir, c.Name))
		if err != nil {
			return fmt.Errorf("parse %s: %w", c.Name, err)
		}
		if len(posts) == 0 {
			continue
		}
		if err := lib.WritePage(distDir, filepath.Join(c.Name, "index.html"), c.Index(posts)); err != nil {
			return fmt.Errorf("render %s index: %w", c.Name, err)
		}
		for _, p := range posts {
			if err := lib.WritePage(distDir, p.OutputPath(), c.Detail(p)); err != nil {
				return fmt.Errorf("render %s/%s: %w", c.Name, p.Slug, err)
			}
		}
		latest = append(latest, posts...)
	}

	topPosts, err := lib.ParseDir("", contentDir)
	if err != nil {
		return fmt.Errorf("parse content root: %w", err)
	}
	for _, p := range topPosts {
		if s.Pages == nil {
			continue
		}
		render, ok := s.Pages[p.Slug]
		if !ok || render == nil {
			continue
		}
		if err := lib.WritePage(distDir, p.OutputPath(), render(p)); err != nil {
			return fmt.Errorf("render %s: %w", p.Slug, err)
		}
	}

	if s.Index != nil {
		sort.Slice(latest, func(i, j int) bool {
			if !latest[i].Date.Equal(latest[j].Date) {
				return latest[i].Date.After(latest[j].Date)
			}
			return latest[i].Title < latest[j].Title
		})
		if err := lib.WritePage(distDir, "index.html", s.Index(latest)); err != nil {
			return fmt.Errorf("render index: %w", err)
		}
	}

	if err := lib.CopyTree(publicDir, distDir); err != nil {
		return fmt.Errorf("copy public: %w", err)
	}
	return nil
}

func Serve(distDir, port string) error {
	return lib.ServeDir(distDir, port)
}

func main() {
	root := flag.String("root", ".", "project root containing src/pages/, src/content/, public/, dist/")
	serve := flag.Bool("serve", false, "start a static file server after generating")
	port := flag.String("port", "3000", "port for static server (with --serve)")
	flag.Parse()

	site := Site{
		Root:        *root,
		Index:       pages.Index,
		Pages:       map[string]func(p lib.Post) templ.Component{"legal": pages.Legal},
		Catalogs: []Catalog{
			{Name: "posts", Index: posts.Page, Detail: posts.Post},
		},
	}

	if err := Build(site); err != nil {
		log.Fatalf("build: %v", err)
	}
	distDir := filepath.Join(*root, "dist")
	log.Printf("wrote site to %s", distDir)

	if *serve {
		if err := Serve(distDir, *port); err != nil {
			log.Fatalf("serve: %v", err)
		}
	}
}
