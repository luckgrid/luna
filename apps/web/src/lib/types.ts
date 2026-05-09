/**
 * Shared types for the static site (content model, view models, templates).
 */

/** Parsed page from Markdown + frontmatter (see `content/parse.ts`). */
export interface Post {
  section: string;
  slug: string;
  category: string;
  title: string;
  description: string;
  date: Date;
  tags: string[];
  html: string;
  /** From frontmatter; used to pick Handlebars layout when set. */
  layout?: string;
  /** Home only: heading above the featured posts list. */
  latestPostsTitle?: string;
}

export interface BuildOptions {
  root: string;
}

/** Handlebars view models */
export interface PostListItemVm {
  url: string;
  title: string;
  hasDate: boolean;
  dateYmd: string;
  dateDisplay: string;
  description?: string;
}

export interface PageHeaderVm {
  caption: string;
  title: string;
  description: string;
}

export interface ArticleMetaVm {
  hasDate: boolean;
  dateYmd: string;
  dateDisplay: string;
  hasTags: boolean;
  tags: string[];
}

export interface CatalogSectionVm {
  title: string;
  description?: string;
  items: PostListItemVm[];
}

/** Handlebars instance from `Handlebars.create()` (narrow surface for `render.ts`). */
export interface HandlebarsEnv {
  compile: (src: string) => (context: unknown) => string;
  partials: Record<string, unknown>;
}

/** Loaded Handlebars + document wrapper */
export interface SiteTemplates {
  hb: HandlebarsEnv;
  base: (context: Record<string, unknown>) => string;
}

export interface BaseVars {
  title: string;
  description: string;
  layout: string;
  pattern: string;
  main: string;
}
