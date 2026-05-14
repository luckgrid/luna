---
title: '{{ replace .File.ContentBaseName "-" " " | title }}'
description: ""
draft: false
type: catalog
params:
  layout: catalog
  pattern: catalog
cascade:
  type: article
  params:
    layout: article
    pattern: post
---

Intro copy for this **catalog** page (list + search + pagination). Section children render below when you use the catalog section layout.
