import { Accordion } from "@luna/ui/accordion";
import { Button } from "@luna/ui/button";
import { Input } from "@luna/ui/input";
import { Link } from "@luna/ui/link";
import { Tooltip } from "@luna/ui/tooltip";
import { Title } from "@solidjs/meta";

export default function Home() {
  return (
    <main>
      <Title>Hello World</Title>
      <div class="mx-auto flex w-full max-w-2xl flex-col gap-6 px-4 py-8 text-left">
        <h1 class="m-0 text-4xl font-semibold text-slate-900">Luna UI + DS integration</h1>

        <div class="flex flex-wrap items-center gap-3">
          <Button>Primary action</Button>
          <Button class="bg-gray-200 text-gray-900 hover:bg-gray-300">Secondary</Button>
          <Tooltip content="This is a custom tooltip from @luna/ui">
            <Button class="h-8 bg-transparent px-3 text-gray-900 hover:bg-gray-100">
              Hover me
            </Button>
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

        <p class="m-0 text-sm text-slate-700">
          <Link href="https://start.solidjs.com" target="_blank" rel="noopener noreferrer">
            SolidStart docs
          </Link>{" "}
          and shared packages now work together.
        </p>
      </div>
    </main>
  );
}
