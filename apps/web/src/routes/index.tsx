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
      <div class="stack">
        <h1>Luna UI + DS integration</h1>

        <div class="cluster">
          <Button>Default</Button>
          <Button class="primary">Primary</Button>
          <Button class="secondary">Secondary</Button>
          <Tooltip content="This is a custom tooltip from @luna/ui">
            <Button class="ghost">Ghost</Button>
          </Tooltip>
        </div>

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
                "Each app imports @luna/ds/styles.css from its own app.css and can layer local overrides there.",
            },
          ]}
        />

        <div class="cluster">
          <span>Scoped component test:</span>
          <Counter />
        </div>

        <p>
          <Link href="https://start.solidjs.com" target="_blank" rel="noopener noreferrer">
            SolidStart docs
          </Link>{" "}
          and shared packages now work together.
        </p>
      </div>
    </main>
  );
}
