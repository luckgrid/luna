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
