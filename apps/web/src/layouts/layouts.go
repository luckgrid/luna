package layouts

// BaseProps configures the HTML shell rendered by [Base].
// Field order in struct literals does not matter. Omit Layout for non-article
// pages (empty string); set Layout to "article" for long-form article CSS.
type BaseProps struct {
	Title       string
	Description string
	Layout      string
	Pattern     string
}
