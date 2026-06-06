# Luna Tech Debt Roadmap

A documented list of tech debt notes, details, etc... to serve as a roadmap for paying back the tech debt before public release.

## Critical tech debt to solve before production

- Config files, outputs, and cache for various toolchain package managers in the root should be contained inside of a .luna/ directory (similar to .moon) and managed by the luna cli. Reduce the number of cli commands down to the most basic ones (install, build, dev, run, clean, check, fix, outdated, update, etc)
- A permanent docs/ workspace/dir should include a collection of docs with detailed content that's rendered as a documentation site (using existing tools and packages - can be a hugo ssg like the web app)
- Create actual useful packages that can be used by go (hugo apps) and py (python apps) in a similar way as the ui and ds packages are used. The go package can contain common layouts, content, and other configs/patterns. The python package can contain backend patterns/tools/configs for pydantic ai, storage, data models, etc.
- Setup an env variable management tool for encryption/decryption of env vars, key vaults, etc (a low level rust tool preferred, use an open source solution or combine open source tools/crates to create a minimal solution that integrates with the cli, or use something like dotenvx which works with multiple toolchains as well).
  - https://docs.rs/secret-vault/latest/secret_vault/
  - https://github.com/Tongsuo-Project/RustyVault
  - https://github.com/Infisical/infisical-cli
  - https://github.com/dotenvx/dotenvx

## Complex tech debt to refactor with deeper research and planning

- Update ds package to dist or provide ds tailwind styles/configs that don't depend on vite and ts (ds should work with hugo/go apps without needing to add package.json and tsconfig in order to run tailwind cli -- ds package can have a build/dev script that can serve the necessary css files to apps, to remove tailwind/ts as a required dependency for using ds) -- can also add/plan tree shaking to update css imports/sources based on what the app needs/uses to reduce the css file size/length
- Update ui package to have reusable html go template layouts, partials, etc (to share between web and future docs apps). ui package should also dist the necessary ui components and markup (with tree shaking of unused components/layouts based on each app) to make the ui usable apps running different stacks and toolchains (i.e. solid/ts, hugo/go, etc) -- integrating the ui with the cli (or low level rust code) to transform/compile ds primitives and ui templates/configs into reusable components, templates, partials, layouts, etc based on which app/package is consuming/using it (i.e. tsx -> solidjs/react/etc, html -> html go, templ, handlebars, etc).
- Implement low level build and dependency management tools using existing libs to improve cli package and multi-language configs
  - https://github.com/prefix-dev/pixi
    - https://pixi.prefix.dev/latest/
    - https://prefix.dev/tools/pixi
    - https://prefix.dev/tools/rattler-build
  - https://gittup.org/tup/
  - https://vboussange.github.io/post/research-project-dependencies/
