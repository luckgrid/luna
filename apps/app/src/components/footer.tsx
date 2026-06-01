import { Navigation } from "./navigation";

const year = new Date().getFullYear();

export function Footer() {
  return (
    <footer>
      <Navigation label="Footer navigation" />
      <small>&copy; {year} Luna</small>
    </footer>
  );
}
