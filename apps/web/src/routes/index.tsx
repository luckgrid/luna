import { Accordion } from "@luna/ui/accordion";
import { Button } from "@luna/ui/button";
import { Input } from "@luna/ui/input";
import { Link } from "@luna/ui/link";
import { Tooltip } from "@luna/ui/tooltip";
import { Title } from "@solidjs/meta";
import Counter from "~/components/counter";

export default function Home() {
  return (
    <main>
      <Title>Hello World</Title>
      <header>
        <h1>Luna UI + DS integration</h1>

        <div role="toolbar" aria-label="Button examples">
          <Button>Default</Button>
          <Button class="button-primary">Primary</Button>
          <Button class="button-secondary">Secondary</Button>
          <Tooltip content="This is a custom tooltip from @luna/ui">
            <Button class="button-ghost">Ghost</Button>
          </Tooltip>
        </div>
      </header>

      <section>
        <Input
          label="Email"
          placeholder="you@example.com"
          hint="We only use this for product updates."
        />

        <Accordion
          items={[
            {
              title: "Why this package split?",
              content:
                "Apps share reusable components through @luna/ui and design tokens/styles through @luna/ds.",
            },
            {
              title: "How do apps override styles?",
              content:
                "Each app imports @luna/ds/tailwind.css from its own app.css and can layer local overrides there.",
            },
          ]}
        />

        <div role="toolbar" aria-label="Counter controls">
          <span>Scoped component test:</span>
          <Counter />
        </div>
      </section>

      <footer>
        <p>
          <Link href="https://start.solidjs.com" target="_blank" rel="noopener noreferrer">
            SolidStart docs
          </Link>{" "}
          and shared packages now work together.
        </p>
      </footer>
    </main>
  );
}
