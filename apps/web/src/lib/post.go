// Package lib provides the content parsing building blocks: a typed Post
// derived from frontmatter and a Markdown renderer, plus basic taxonomy
// helpers for grouping and sorting posts.
package lib

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/yuin/goldmark"
	meta "github.com/yuin/goldmark-meta"
	"github.com/yuin/goldmark/parser"
	"github.com/yuin/goldmark/renderer/html"
)

// Post is a single Markdown document parsed into typed fields.
//
//   - Section names the catalog folder under src/content/ ("posts").
//     Empty for top-level Markdown files (like src/content/legal.md).
//   - Slug is derived from the filename (without `.md`) unless overridden
//     via the optional `slug:` frontmatter field.
//   - Category is set by the `category:` frontmatter field; it is used
//     for grouping in catalog indexes but never appears in URLs.
type Post struct {
	Section     string
	Slug        string
	Category    string
	Title       string
	Description string
	Date        time.Time
	Tags        []string
	HTML        string
}

// URL returns the canonical site path for the post.
//
// Top-level pages (Section == "") become /<slug>.html. Posts inside a
// catalog become /<section>/<slug>/.
func (p Post) URL() string {
	if p.Section == "" {
		return "/" + p.Slug + ".html"
	}
	return "/" + p.Section + "/" + p.Slug + "/"
}

// OutputPath returns the dist-relative file path for the post.
func (p Post) OutputPath() string {
	if p.Section == "" {
		return p.Slug + ".html"
	}
	return filepath.Join(p.Section, p.Slug, "index.html")
}

// Eyebrow returns the small label rendered above a post's title — its
// category if set, otherwise its section.
func (p Post) Eyebrow() string {
	if p.Category != "" {
		return p.Category
	}
	return p.Section
}

var md = goldmark.New(
	goldmark.WithExtensions(meta.Meta),
	goldmark.WithParserOptions(parser.WithAutoHeadingID()),
	goldmark.WithRendererOptions(html.WithUnsafe()),
)

// ParseFile reads a single Markdown file and returns the parsed Post.
// Slug defaults to the filename without `.md`; pass section="" for
// top-level pages.
func ParseFile(section, path string) (Post, error) {
	raw, err := os.ReadFile(path)
	if err != nil {
		return Post{}, err
	}

	var buf bytes.Buffer
	ctx := parser.NewContext()
	if err := md.Convert(raw, &buf, parser.WithContext(ctx)); err != nil {
		return Post{}, fmt.Errorf("render %s: %w", path, err)
	}

	fm := meta.Get(ctx)
	slug := strings.TrimSuffix(filepath.Base(path), ".md")
	if v, ok := fm["slug"].(string); ok && v != "" {
		slug = v
	}

	title := slug
	if v, ok := fm["title"].(string); ok && v != "" {
		title = v
	}

	category, _ := fm["category"].(string)
	description, _ := fm["description"].(string)

	var date time.Time
	if v, ok := fm["date"].(string); ok && v != "" {
		if t, err := time.Parse("2006-01-02", v); err == nil {
			date = t
		}
	}

	var tags []string
	if v, ok := fm["tags"].([]any); ok {
		for _, t := range v {
			if s, ok := t.(string); ok {
				tags = append(tags, s)
			}
		}
	}

	return Post{
		Section:     section,
		Slug:        slug,
		Category:    category,
		Title:       title,
		Description: description,
		Date:        date,
		Tags:        tags,
		HTML:        buf.String(),
	}, nil
}

// ParseDir reads top-level .md files in dir (non-recursive). It returns
// posts sorted newest-first via SortNewestFirst.
func ParseDir(section, dir string) ([]Post, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, nil
		}
		return nil, err
	}

	var posts []Post
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".md") {
			continue
		}
		p, err := ParseFile(section, filepath.Join(dir, e.Name()))
		if err != nil {
			return nil, err
		}
		posts = append(posts, p)
	}

	SortNewestFirst(posts)
	return posts, nil
}

// SortNewestFirst sorts posts by date (newest-first), breaking ties by
// title. This matches the ordering used for catalogs and the home page.
func SortNewestFirst(posts []Post) {
	sort.Slice(posts, func(i, j int) bool {
		if !posts[i].Date.Equal(posts[j].Date) {
			return posts[i].Date.After(posts[j].Date)
		}
		return posts[i].Title < posts[j].Title
	})
}

// GroupByCategory groups posts by their Category field, preserving input
// order within each group. The "" key holds posts with no category.
func GroupByCategory(posts []Post) map[string][]Post {
	out := map[string][]Post{}
	for _, p := range posts {
		out[p.Category] = append(out[p.Category], p)
	}
	return out
}

// Categories returns the unique, sorted list of non-empty categories
// used across the given posts.
func Categories(posts []Post) []string {
	seen := map[string]bool{}
	var out []string
	for _, p := range posts {
		if p.Category == "" || seen[p.Category] {
			continue
		}
		seen[p.Category] = true
		out = append(out, p.Category)
	}
	sort.Strings(out)
	return out
}
