import { Title } from "@solidjs/meta";

export default function About() {
  return (
    <main>
      <Title>About | Luna</Title>
      <header>
        <hgroup>
          <h1>Luna</h1>
          <p>DX first, AI ready, fast, minimal, and powerful monorepo starter template.</p>
        </hgroup>
      </header>
      <section>
        <p>
          Luna is a fast, modern, and flexible monorepo starter template. It comes with a CSS-first,
          class-less design system built on top of Tailwind CSS, a minimal SolidJS UI library that
          abstracts UI/UX patterns and an AI Agent framework for building web apps with generative
          AI features.
        </p>
      </section>
    </main>
  );
}
