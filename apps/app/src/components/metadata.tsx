import { Link, Meta, Title } from "@solidjs/meta";
import { useCurrentMatches, useLocation } from "@solidjs/router";
import { createMemo, splitProps } from "solid-js";

import { NOT_FOUND_SEO, ROUTE_SEO, SEO_DEFAULTS, type RouteSeo } from "~/app.config";
import { isNotFoundPath, joinCanonicalUrl, normalizePathname } from "~/utils/url";

export type MetadataProps = {
  title: string;
  description?: string;
  appName?: string;
  baseUrl?: string;
  locale?: string;
  canonicalPath?: string;
  image?: string;
  noIndex?: boolean;
};

function routeMetadata(pathname: string, isNotFound: boolean): RouteSeo {
  if (isNotFound) {
    return NOT_FOUND_SEO;
  }

  return (
    ROUTE_SEO[pathname] ?? {
      title: SEO_DEFAULTS.defaultTitle,
      description: SEO_DEFAULTS.defaultDescription,
    }
  );
}

export function Metadata(props: MetadataProps) {
  const [local] = splitProps(props, [
    "title",
    "description",
    "appName",
    "baseUrl",
    "locale",
    "canonicalPath",
    "image",
    "noIndex",
  ]);
  const location = useLocation();

  const appName = () => local.appName ?? SEO_DEFAULTS.appName;
  const baseUrl = () => local.baseUrl ?? SEO_DEFAULTS.baseUrl;
  const locale = () => local.locale ?? SEO_DEFAULTS.locale;
  const description = () => local.description ?? SEO_DEFAULTS.defaultDescription;
  const documentTitle = () => `${local.title} | ${appName()}`;
  const canonicalUrl = createMemo(() =>
    joinCanonicalUrl(baseUrl(), local.canonicalPath ?? normalizePathname(location.pathname)),
  );
  const imageUrl = createMemo(() => {
    const image = local.image;
    if (!image) return undefined;
    if (image.startsWith("http://") || image.startsWith("https://")) {
      return image;
    }
    return joinCanonicalUrl(baseUrl(), image.startsWith("/") ? image : `/${image}`);
  });

  return (
    <>
      <Title>{documentTitle()}</Title>
      <Meta name="description" content={description()} />
      <Meta property="og:title" content={documentTitle()} />
      <Meta property="og:description" content={description()} />
      <Meta property="og:url" content={canonicalUrl()} />
      <Meta property="og:type" content={SEO_DEFAULTS.ogType} />
      <Meta property="og:locale" content={locale()} />
      <Meta name="twitter:card" content={SEO_DEFAULTS.twitterCard} />
      <Meta name="twitter:title" content={documentTitle()} />
      <Meta name="twitter:description" content={description()} />
      <Link rel="canonical" href={canonicalUrl()} />
      {imageUrl() && <Meta property="og:image" content={imageUrl()} />}
      {local.noIndex && <Meta name="robots" content="noindex, nofollow" />}
    </>
  );
}

/** Route-aware metadata rendered once at the app shell (see `ROUTE_SEO` in `~/app.config`). */
export function Seo() {
  const location = useLocation();
  const matches = useCurrentMatches();

  const meta = createMemo(() => {
    const pathname = normalizePathname(location.pathname);
    const leaf = matches().at(-1);
    return routeMetadata(pathname, isNotFoundPath(pathname, leaf));
  });

  return <Metadata {...meta()} />;
}
