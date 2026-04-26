import { ButtonLink } from "@luna/ui/button";
import { Link } from "@luna/ui/link";
import { Title } from "@solidjs/meta";

export default function Home() {
  return (
    <main>
      <Title>Monorepo Starter Template | Luna</Title>
      <header>
        <hgroup>
          <h1>Luna</h1>
          <p>A Moonrepo starter template using Bun, SolidStart, and Solid Router.</p>
        </hgroup>
        <div role="toolbar" aria-label="Button examples">
          <ButtonLink href="https://github.com/luckgrid/luna">Get Started</ButtonLink>
        </div>
      </header>

      <section>
        <h2>Features</h2>
        <p>
          Luna is a monorepo starter template that provides a foundation for building web apps with
          SolidJS. It includes a design system, UI components, and a chatbot powered by Pydantic AI
          and FastAPI.
        </p>
      </section>

      <footer>
        <small>
          Built with <Link href="https://github.com/luckgrid/luna">Luna</Link>.
        </small>
      </footer>
    </main>
  );
}
