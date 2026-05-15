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
