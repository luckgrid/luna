import { createSignal } from "solid-js";

export default function Counter() {
  const [count, setCount] = createSignal(0);
  return (
    <button
      class="w-[200px] cursor-pointer rounded-full border-2 border-transparent bg-[rgba(68,107,158,0.1)] px-8 py-4 font-inherit [font-size:inherit] tabular-nums text-[#335d92] outline-none focus:border-[#335d92] active:bg-[rgba(68,107,158,0.2)]"
      onClick={() => setCount(count() + 1)}
      type="button"
    >
      Clicks: {count()}
    </button>
  );
}
