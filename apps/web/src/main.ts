import path from "node:path";

import { contentDir, outputPath } from "./lib/content/utils";
import { parsePostsFolder, parseTopLevelPages, sortNewestFirst } from "./lib/content/parse";
import type { BuildOptions, Post } from "./lib/types";
import {
  loadTemplates,
  renderPageArticle,
  renderPageCatalog,
  renderPageDefault,
} from "./lib/templates/render";
import { copyTree, syntheticCatalog, syntheticHome, writePage } from "./lib/utils";

export async function main(opts: BuildOptions): Promise<void> {
  const { root } = opts;
  const cdir = contentDir(root);
  const distDir = path.join(root, "dist");
  const publicDir = path.join(root, "public");

  const tpl = await loadTemplates(root);

  const { home: homeRaw, pages: topPages } = await parseTopLevelPages(cdir);
  const { posts, catalog } = await parsePostsFolder(path.join(cdir, "posts"));

  const latest: Post[] = [...posts];
  sortNewestFirst(latest);

  const home = homeRaw ?? syntheticHome();
  const catalogPage = catalog ?? (posts.length > 0 ? syntheticCatalog() : null);

  const latestTitle = home.latestPostsTitle ?? "Latest posts";

  await writePage(distDir, "index.html", renderPageDefault(tpl, home, latest, latestTitle));

  if (catalogPage !== null) {
    await writePage(
      distDir,
      path.join("posts", "index.html"),
      renderPageCatalog(tpl, catalogPage, posts),
    );
  }

  await Promise.all(
    posts.map((p) =>
      writePage(distDir, outputPath(p), renderPageArticle(tpl, p, { pattern: "post" })),
    ),
  );

  await Promise.all(
    topPages.map((p) => writePage(distDir, outputPath(p), renderPageArticle(tpl, p, {}))),
  );

  await copyTree(publicDir, distDir);

  console.log(`wrote site to ${distDir}`);
}

function parseArgs(): BuildOptions {
  const argv = process.argv.slice(2);
  let root = process.cwd();
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--root" && argv[i + 1] !== undefined) {
      root = path.resolve(argv[i + 1]);
      i++;
    }
  }
  return { root };
}

if (import.meta.main) {
  const opts = parseArgs();
  await main(opts);
}
