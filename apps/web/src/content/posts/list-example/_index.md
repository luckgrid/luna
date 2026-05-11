---
title: Example collection
description: Nested posts that use the same collection-layout chrome as Legal — sidebar nav, filter search, and optional TOC column.
weight: 5
toc: true
cascade:
  params:
    collection_sidebar_search_label: "Search this collection"
    collection_nav_aria: "Example collection pages"
---

This section lives under [`posts/list-example/`](.) and demonstrates the **collection** layout shared with [Legal](/legal/): left sidebar with filter-as-you-type search, main column, and optional right-hand TOC when `toc` is true.

Scaffold **article hubs** with **`hugo new content …/_index.md -k article-hub`** ([`article-hub.md`](../../../archetypes/article-hub.md)). Scaffold **pages inside the hub** with **`hugo new content posts/list-example/<slug>.md -k article`** ([`article.md`](../../../archetypes/article.md); [Hugo archetypes](https://gohugo.io/content-management/archetypes/)).
