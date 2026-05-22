import { HttpStatusCode } from "@solidjs/start";

import { Hero } from "~/components/hero";

export default function NotFound() {
  return (
    <main>
      <HttpStatusCode code={404} />
      <Hero title="Page Not Found" />
      <p>
        Visit{" "}
        <a href="https://start.solidjs.com" target="_blank" rel="noopener noreferrer">
          start.solidjs.com
        </a>{" "}
        to learn how to build SolidStart apps.
      </p>
    </main>
  );
}
