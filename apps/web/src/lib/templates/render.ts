/**
 * Load `src/templates` HTML (Handlebars) and compose final pages.
 */
import Handlebars from "handlebars";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import type { BaseVars, Post, SiteTemplates } from "../types";
import { caption } from "../content/utils";
import {
  buildArticleMeta,
  buildCatalogSections,
  buildLatestItems,
  buildPageHeader,
} from "./context";

const ENABLE_DATASTAR = false;
const DATASTAR_SCRIPT_SRC =
  "https://cdn.jsdelivr.net/gh/starfederation/datastar@1.0.0/bundles/datastar.js";

export async function loadTemplates(root: string): Promise<SiteTemplates> {
  const tplRoot = path.join(root, "src", "templates");
  const hb = Handlebars.create();

  hb.registerHelper("eq", (a: unknown, b: unknown) => a === b);
  hb.registerHelper("or", (...args: unknown[]) => {
    const values = args.slice(0, -1);
    return values.some(Boolean);
  });

  const partialDir = path.join(tplRoot, "partials");
  try {
    const partialFiles = (await readdir(partialDir)).filter((n) => n.endsWith(".html"));
    await Promise.all(
      partialFiles.map(async (name) => {
        const key = path.basename(name, ".html");
        const src = await readFile(path.join(partialDir, name), "utf8");
        hb.registerPartial(key, src);
      }),
    );
  } catch {
    /* no partials */
  }

  const layoutsDir = path.join(tplRoot, "layouts");
  const layoutFiles = (await readdir(layoutsDir)).filter(
    (n) => n.endsWith(".html") && n !== "base.html",
  );
  await Promise.all(
    layoutFiles.map(async (name) => {
      const key = path.basename(name, ".html");
      const src = await readFile(path.join(layoutsDir, name), "utf8");
      hb.registerPartial(`layout-${key}`, src);
    }),
  );

  const basePath = path.join(tplRoot, "layouts", "base.html");
  const baseSrc = await readFile(basePath, "utf8");
  const base = hb.compile(baseSrc);

  return { hb, base };
}

export function renderBase(tpl: SiteTemplates, vars: BaseVars): string {
  const datastar = ENABLE_DATASTAR
    ? `<script type="module" src="${Handlebars.escapeExpression(DATASTAR_SCRIPT_SRC)}"></script>`
    : "";
  return tpl.base({
    title: vars.title,
    description: vars.description,
    layout: vars.layout,
    pattern: vars.pattern,
    datastar,
    main: vars.main,
  });
}

export function renderLayout(
  tpl: SiteTemplates,
  layoutName: string,
  context: Record<string, unknown>,
): string {
  const key = `layout-${layoutName}`;
  const raw = tpl.hb.partials[key];
  const src = typeof raw === "string" ? raw : "";
  if (src === "") {
    throw new Error(`Missing template partial: ${key}`);
  }
  return tpl.hb.compile(src)(context);
}

export function renderPageDefault(
  tpl: SiteTemplates,
  page: Post,
  latest: Post[],
  latestTitle: string,
): string {
  const main = renderLayout(tpl, "default", {
    title: page.title,
    description: page.description,
    meta: buildArticleMeta(page),
    body: page.html,
    latestTitle,
    latestItems: buildLatestItems(latest),
  });
  return renderBase(tpl, {
    title: page.title,
    description: page.description,
    layout: "default",
    pattern: "",
    main,
  });
}

export function renderPageCatalog(tpl: SiteTemplates, page: Post, posts: Post[]): string {
  const main = renderLayout(tpl, "catalog", {
    category: page.category,
    title: page.title,
    description: page.description,
    meta: buildArticleMeta(page),
    body: page.html,
    catalogSections: buildCatalogSections(posts),
  });
  return renderBase(tpl, {
    title: page.title,
    description: page.description,
    layout: "catalog",
    pattern: "",
    main,
  });
}

export function renderPageArticle(
  tpl: SiteTemplates,
  page: Post,
  opts: { pattern?: string } = {},
): string {
  const pageHeader = buildPageHeader(caption(page), page.title, page.description);
  const meta = buildArticleMeta(page);
  const main = renderLayout(tpl, "article", {
    pageHeader,
    meta,
    body: page.html,
  });
  return renderBase(tpl, {
    title: page.title,
    description: page.description,
    layout: "article",
    pattern: opts.pattern ?? "",
    main,
  });
}
