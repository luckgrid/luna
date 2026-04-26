import { Title } from "@solidjs/meta";
import { Show, createSignal } from "solid-js";
import { Link } from "@luna/ui/link";

export default function DesignSystemPage() {
  const [isModalOpen, setIsModalOpen] = createSignal(false);

  return (
    <main>
      <Title>Design System | Luna</Title>
      <header>
        <hgroup>
          <h1>Luna Design System</h1>
          <p>
            The design system provides class-less CSS-first primitives for building consistent UI
            without dependencies, only HTML primitives.
          </p>
        </hgroup>
      </header>

      <section id="preview">
        <h2>Preview</h2>
        <p>
          Sed ultricies dolor non ante vulputate hendrerit. Vivamus sit amet suscipit sapien. Nulla
          iaculis eros a elit pharetra egestas.
        </p>
        <form onSubmit={(event) => event.preventDefault()}>
          <input
            type="text"
            name="firstname"
            placeholder="First name"
            aria-label="First name"
            required
          />
          <input
            type="email"
            name="email"
            placeholder="Email address"
            aria-label="Email address"
            autocomplete="email"
            required
          />
          <button type="submit">Subscribe</button>
          <fieldset>
            <label for="terms">
              <input type="checkbox" role="switch" id="terms" name="terms" />I agree to the{" "}
              <a href="#" onClick={(event) => event.preventDefault()}>
                Privacy Policy
              </a>
            </label>
          </fieldset>
        </form>
      </section>

      <section id="typography">
        <h2>Typography</h2>
        <p>
          Aliquam lobortis vitae nibh nec rhoncus. Morbi mattis neque eget efficitur feugiat.
          Vivamus porta nunc a erat mattis, mattis feugiat turpis pretium. Quisque sed tristique
          felis.
        </p>
        <blockquote>
          "Maecenas vehicula metus tellus, vitae congue turpis hendrerit non. Nam at dui sit amet
          ipsum cursus ornare."
          <footer>
            <cite>- Phasellus eget lacinia</cite>
          </footer>
        </blockquote>
        <h3>Lists</h3>
        <ul>
          <li>Aliquam lobortis lacus eu libero ornare facilisis.</li>
          <li>Nam et magna at libero scelerisque egestas.</li>
          <li>Suspendisse id nisl ut leo finibus vehicula quis eu ex.</li>
          <li>Proin ultricies turpis et volutpat vehicula.</li>
        </ul>
        <h3>Inline text elements</h3>
        <p>
          <a href="#" onClick={(event) => event.preventDefault()}>
            Link
          </a>
        </p>
        <p>
          <strong>Bold</strong>
        </p>
        <p>
          <em>Italic</em>
        </p>
        <p>
          <u>Underline</u>
        </p>
        <p>
          <del>Deleted</del>
        </p>
        <p>
          <ins>Inserted</ins>
        </p>
        <p>
          <s>Strikethrough</s>
        </p>
        <p>
          <small>Small</small>
        </p>
        <p>
          Text <sub>Sub</sub>
        </p>
        <p>
          Text <sup>Sup</sup>
        </p>
        <p>
          <abbr title="Abbreviation" data-tooltip="Abbreviation">
            Abbr.
          </abbr>
        </p>
        <p>
          <kbd>Kbd</kbd>
        </p>
        <p>
          <mark>Highlighted</mark>
        </p>
        <h3>Heading 3</h3>
        <p>
          Integer bibendum malesuada libero vel eleifend. Fusce iaculis turpis ipsum, at efficitur
          sem scelerisque vel. Aliquam auctor diam ut purus cursus fringilla. Class aptent taciti
          sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos.
        </p>
        <h4>Heading 4</h4>
        <p>
          Cras fermentum velit vitae auctor aliquet. Nunc non congue urna, at blandit nibh. Donec ac
          fermentum felis. Vivamus tincidunt arcu ut lacus hendrerit, eget mattis dui finibus.
        </p>
        <h5>Heading 5</h5>
        <p>
          Donec nec egestas nulla. Sed varius placerat felis eu suscipit. Mauris maximus ante in
          consequat luctus. Morbi euismod sagittis efficitur. Aenean non eros orci. Vivamus ut diam
          sem.
        </p>
        <h6>Heading 6</h6>
        <p>
          Ut sed quam non mauris placerat consequat vitae id risus. Vestibulum tincidunt nulla ut
          tortor posuere, vitae malesuada tortor molestie. Sed nec interdum dolor. Vestibulum id
          auctor nisi, a efficitur sem. Aliquam sollicitudin efficitur turpis, sollicitudin
          hendrerit ligula semper id. Nunc risus felis, egestas eu tristique eget, convallis in
          velit.
        </p>
        <figure>
          <img
            src="https://images.unsplash.com/photo-1506744038136-46273834b3fb?auto=format&fit=crop&w=2000&q=80"
            alt="Minimal landscape"
          />
          <figcaption>
            Image from{" "}
            <a
              href="https://unsplash.com/photos/a562ZEFKW8I"
              target="_blank"
              rel="noopener noreferrer"
            >
              unsplash.com
            </a>
          </figcaption>
        </figure>
      </section>

      <section id="form">
        <form onSubmit={(event) => event.preventDefault()}>
          <h2>Form elements</h2>
          <label for="search">Search</label>
          <input type="search" id="search" name="search" placeholder="Search" />
          <label for="text">Text</label>
          <input type="text" id="text" name="text" placeholder="Text" />
          <small>Curabitur consequat lacus at lacus porta finibus.</small>
          <label for="select">Select</label>
          <select id="select" name="select" required>
            <option value="" selected>
              Select...
            </option>
            <option>...</option>
          </select>
          <label for="file">
            File browser
            <input type="file" id="file" name="file" />
          </label>
          <label for="range">
            Range slider
            <input type="range" min="0" max="100" value="50" id="range" name="range" />
          </label>
          <label for="valid">
            Valid
            <input type="text" id="valid" name="valid" placeholder="Valid" aria-invalid="false" />
          </label>
          <label for="invalid">
            Invalid
            <input
              type="text"
              id="invalid"
              name="invalid"
              placeholder="Invalid"
              aria-invalid="true"
            />
          </label>
          <label for="disabled">
            Disabled
            <input type="text" id="disabled" name="disabled" placeholder="Disabled" disabled />
          </label>
          <label for="date">
            Date
            <input type="date" id="date" name="date" />
          </label>
          <label for="time">
            Time
            <input type="time" id="time" name="time" />
          </label>
          <label for="color">
            Color
            <input type="color" id="color" name="color" value="#0eaaaa" />
          </label>
          <fieldset>
            <legend>
              <strong>Checkboxes</strong>
            </legend>
            <label for="checkbox-1">
              <input type="checkbox" id="checkbox-1" name="checkbox-1" checked />
              Checkbox
            </label>
            <label for="checkbox-2">
              <input type="checkbox" id="checkbox-2" name="checkbox-2" />
              Checkbox
            </label>
          </fieldset>
          <fieldset>
            <legend>
              <strong>Radio buttons</strong>
            </legend>
            <label for="radio-1">
              <input type="radio" id="radio-1" name="radio" value="radio-1" checked />
              Radio button
            </label>
            <label for="radio-2">
              <input type="radio" id="radio-2" name="radio" value="radio-2" />
              Radio button
            </label>
          </fieldset>
          <fieldset>
            <legend>
              <strong>Switches</strong>
            </legend>
            <label for="switch-1">
              <input type="checkbox" id="switch-1" name="switch-1" role="switch" checked />
              Switch
            </label>
            <label for="switch-2">
              <input type="checkbox" id="switch-2" name="switch-2" role="switch" />
              Switch
            </label>
          </fieldset>
          <input type="reset" value="Reset" />
          <input type="submit" value="Submit" />
        </form>
      </section>

      <section id="modal">
        <h2>Modal</h2>
        <button class="contrast" onClick={() => setIsModalOpen(true)}>
          Launch demo modal
        </button>
      </section>

      <section id="accordions">
        <h2>Accordions</h2>
        <details>
          <summary>Accordion 1</summary>
          <p data-content>
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pellentesque urna diam,
            tincidunt nec porta sed, auctor id velit. Etiam venenatis nisl ut orci consequat, vitae
            tempus quam commodo. Nulla non mauris ipsum. Aliquam eu posuere orci. Nulla convallis
            lectus rutrum quam hendrerit, in facilisis elit sollicitudin. Mauris pulvinar pulvinar
            mi, dictum tristique elit auctor quis. Maecenas ac ipsum ultrices, porta turpis sit
            amet, congue turpis.
          </p>
        </details>
        <details open>
          <summary>Accordion 2</summary>
          <ul data-content>
            <li>Vestibulum id elit quis massa interdum sodales.</li>
            <li>Nunc quis eros vel odio pretium tincidunt nec quis neque.</li>
            <li>Quisque sed eros non eros ornare elementum.</li>
            <li>Cras sed libero aliquet, porta dolor quis, dapibus ipsum.</li>
          </ul>
        </details>
      </section>

      <section id="article">
        <h2>Article</h2>
        <article>
          <header>
            <h3>Lorem ipsum dolor sit amet</h3>
          </header>
          <section>
            <h4>Lorem ipsum dolor sit amet</h4>
            <p>
              Nullam dui arcu, malesuada et sodales eu, efficitur vitae dolor. Sed ultricies dolor
              non ante vulputate hendrerit. Vivamus sit amet suscipit sapien. Nulla iaculis eros a
              elit pharetra egestas. Nunc placerat facilisis cursus. Sed vestibulum metus eget dolor
              pharetra rutrum.
            </p>
            <p>
              Sed ultricies dolor non ante vulputate hendrerit. Vivamus sit amet suscipit sapien.
              Nulla iaculis eros a elit pharetra egestas. Nunc placerat facilisis cursus. Sed
              vestibulum metus eget dolor pharetra rutrum.
            </p>
          </section>
          <footer>
            <small>Duis nec elit placerat, suscipit nibh quis, finibus neque.</small>
          </footer>
        </article>
      </section>

      <section id="group">
        <h2>Group</h2>
        <form onSubmit={(event) => event.preventDefault()}>
          <fieldset role="group">
            <input name="email" type="email" placeholder="Enter your email" autocomplete="email" />
            <input type="submit" value="Subscribe" />
          </fieldset>
        </form>
      </section>

      <section id="progress">
        <h2>Progress bar</h2>
        <progress id="progress-1" value="25" max="100"></progress>
        <progress id="progress-2"></progress>
      </section>

      <section id="loading">
        <h2>Loading</h2>
        <article aria-busy="true"></article>
        <button aria-busy="true" type="button">
          Please wait...
        </button>
      </section>

      <footer>
        <small>
          Built with <Link href="https://github.com/luckgrid/luna">Luna</Link> •{" "}
          <Link href="https://github.com/luckgrid/luna/blob/main/apps/web/src/routes/design-system.tsx">
            Source code
          </Link>
        </small>
      </footer>

      <Show when={isModalOpen()}>
        <dialog id="modalExample" open>
          <article>
            <header>
              <h3>Confirm your action!</h3>
              <button aria-label="Close" onClick={() => setIsModalOpen(false)}>
                &times;
              </button>
            </header>
            <section>
              <p>
                Cras sit amet maximus risus. Pellentesque sodales odio sit amet augue finibus
                pellentesque. Nullam finibus risus non semper euismod.
              </p>
            </section>
            <footer>
              <button role="button" onClick={() => setIsModalOpen(false)}>
                Cancel
              </button>
              <button autofocus role="button" onClick={() => setIsModalOpen(false)}>
                Confirm
              </button>
            </footer>
          </article>
        </dialog>
      </Show>
    </main>
  );
}
