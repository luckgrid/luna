export const SEO_DEFAULTS = {
  appName: "Luna",
  defaultTitle: "Luna",
  defaultDescription: "A Moonrepo starter template using Bun, SolidStart, and Solid Router.",
  locale: "en",
  baseUrl: import.meta.env.NITRO_PUBLIC_APP_BASE_URL || "http://localhost:3000",
  ogType: "website",
  twitterCard: "summary",
} as const;

export type RouteSeo = {
  title: string;
  description?: string;
  noIndex?: boolean;
};

export const ROUTE_SEO: Record<string, RouteSeo> = {
  "/": {
    title: "Monorepo Starter Template",
    description: SEO_DEFAULTS.defaultDescription,
  },
  "/ds": {
    title: "Design System",
    description:
      "The design system provides class-less CSS-first primitives for building consistent UI without dependencies, only HTML primitives.",
  },
  "/ui": {
    title: "Solid UI",
    description: "Reusable Solid UI/UX patterns and component.",
  },
  "/ai": {
    title: "Pydantic AI",
    description: "Chat with AI powered by Pydantic AI + FastAPI.",
  },
};

export const NOT_FOUND_SEO: RouteSeo = {
  title: "Not Found",
  noIndex: true,
};

export const KNOWN_PATHS = new Set(Object.keys(ROUTE_SEO));
