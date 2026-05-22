import { ButtonLink } from "@luna/ui/button";

import { Hero } from "~/components/hero";

export default function Home() {
  return (
    <main>
      <Hero
        title="Luna"
        description="A Moonrepo starter template using Bun, SolidStart, and Solid Router."
      >
        <div role="toolbar" aria-label="Button examples">
          <ButtonLink href="https://github.com/luckgrid/luna">Get Started</ButtonLink>
        </div>
      </Hero>

      <section>
        <h2>Features</h2>
        <p>
          Luna is a monorepo starter template that provides a foundation for building web apps with
          SolidJS. It includes a design system, UI components, and a chatbot powered by Pydantic AI
          and FastAPI.
        </p>
      </section>
    </main>
  );
}
