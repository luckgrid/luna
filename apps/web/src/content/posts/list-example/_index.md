---
title: Example collection
description: Nested posts that use the same collection-layout chrome as Legal — sidebar nav, filter search, and optional TOC column.
type: collection
weight: 5
toc: true
params:
  layout: collection
  pattern: posts-list-example
cascade:
  type: collection
  params:
    layout: collection
    pattern: posts-list-example
    collection_sidebar_search_label: "Search this collection"
    collection_nav_aria: "Example collection pages"
---

This section lives under [`posts/list-example/`](.) and demonstrates the **collection** layout shared with [Legal](/legal/): left sidebar with filter-as-you-type search, main column, and optional right-hand TOC when `toc` is true.

Scaffold **collections** with **`hugo new content …/_index.md -k collection`** ([`collection.md`](../../../archetypes/collection.md)). Scaffold **pages inside the collection** with **`hugo new content posts/list-example/<slug>.md -k article`** ([`article.md`](../../../archetypes/article.md); [Hugo archetypes](https://gohugo.io/content-management/archetypes/)).
