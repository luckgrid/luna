---
title: '{{ replace .File.ContentBaseName "-" " " | title }}'
description: ""
type: collection
weight: 1
toc: true
draft: false
params:
  layout: collection
  pattern: collection
cascade:
  type: collection
  params:
    layout: collection
    pattern: collection
    collection_sidebar_search_label: "Filter pages…"
    collection_nav_aria: "Pages in this section"
---

Collection introduction page.

> **Customize per section:** update `params.pattern` (self), `cascade.params.pattern` (children), and the two `collection_*` labels to match the section name (e.g. `legal`, `posts-list-example`).

Add child pages with **`hugo new content … -k article`**.
