/**
 * View models passed to Handlebars (no markup). Dates use `lib/utils` formatting.
 */
import { categories, groupByCategory } from "../content/parse";
import type { ArticleMetaVm, CatalogSectionVm, PageHeaderVm, Post, PostListItemVm } from "../types";
import { postURL } from "../content/utils";
import { formatDisplayDate, formatYmd } from "../utils";

export function toPostListItem(p: Post): PostListItemVm {
  const hasDate = p.date.getTime() !== 0;
  return {
    url: postURL(p),
    title: p.title,
    hasDate,
    dateYmd: formatYmd(p.date),
    dateDisplay: formatDisplayDate(p.date),
    description: p.description !== "" ? p.description : undefined,
  };
}

export function buildLatestItems(posts: Post[]): PostListItemVm[] {
  return posts.map(toPostListItem);
}

export function buildCatalogSections(posts: Post[]): CatalogSectionVm[] {
  if (posts.length === 0) return [];
  const grouped = groupByCategory(posts);
  const sections: CatalogSectionVm[] = [];
  for (const cat of categories(posts)) {
    const list = grouped[cat];
    if (list) sections.push({ title: cat, items: list.map(toPostListItem) });
  }
  const uncategorized = grouped[""];
  if (uncategorized !== undefined && uncategorized.length > 0) {
    sections.push({ title: "Other", items: uncategorized.map(toPostListItem) });
  }
  return sections;
}

export function buildPageHeader(caption: string, title: string, description: string): PageHeaderVm {
  return { caption, title, description };
}

export function buildArticleMeta(p: Post): ArticleMetaVm {
  const hasDate = p.date.getTime() !== 0;
  return {
    hasDate,
    dateYmd: formatYmd(p.date),
    dateDisplay: formatDisplayDate(p.date),
    hasTags: p.tags.length > 0,
    tags: p.tags,
  };
}
