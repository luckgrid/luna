import { Accordion } from "@luna/ui/accordion";
import { Button } from "@luna/ui/button";
import { Input } from "@luna/ui/input";
import Counter from "~/components/counter";
import { Hero } from "~/components/hero";

export default function Home() {
  return (
    <main>
      <Hero title="Luna UI" description="Reusable Solid UI/UX patterns and component.">
        <div role="group" aria-label="Button examples">
          <Button>Default</Button>
          <Button data-button="hollow">Hollow</Button>
          <Button data-button="ghost">Ghost</Button>
          <Button data-button="text">Text</Button>
          <Button data-button="alert">Alert</Button>
          <Button data-button="icon">
            <i data-icon="menu" />
          </Button>
          <Button data-button="wrapper">Wrapper</Button>
        </div>
      </Hero>

      <section>
        <Input aria-label="Email" placeholder="email@example.com" type="email" />

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
    </main>
  );
}
