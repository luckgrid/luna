export function isExternalHref(href?: string): boolean {
  if (!href) {
    return false;
  }

  if (
    href.startsWith("/") ||
    href.startsWith("./") ||
    href.startsWith("../") ||
    href.startsWith("#") ||
    href.startsWith("?")
  ) {
    return false;
  }

  if (href.startsWith("//")) {
    return true;
  }

  return /^[a-zA-Z][a-zA-Z\d+\-.]*:/.test(href);
}

export function withExternalRel(rel?: string): string {
  const values = new Set((rel ?? "").split(/\s+/).filter(Boolean));
  values.add("noopener");
  values.add("noreferrer");
  return Array.from(values).join(" ");
}
