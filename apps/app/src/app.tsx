import { MetaProvider, Title } from "@solidjs/meta";
import { Router } from "@solidjs/router";
import { FileRoutes } from "@solidjs/start/router";
import { Suspense } from "solid-js";

import "./app.css";

export default function App() {
  return (
    <Router
      root={(props) => (
        <MetaProvider>
          <Title>SolidStart - Basic</Title>
          <header>
            <nav>
              <a href="/">Home</a>
              <div class="flex gap-4" aria-label="Main navigation">
                <a href="/design-system">Design System</a>
                <a href="/ui">UI</a>
                <a href="/ai">AI</a>
              </div>
            </nav>
          </header>
          <Suspense>{props.children}</Suspense>
        </MetaProvider>
      )}
    >
      <FileRoutes />
    </Router>
  );
}
