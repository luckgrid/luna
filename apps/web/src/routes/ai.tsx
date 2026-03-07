import { createSignal, For } from "solid-js";
import { Title } from "@solidjs/meta";

type Message = {
  role: "user" | "assistant";
  content: string;
};

export default function AI() {
  const [message, setMessage] = createSignal("");
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    const userMessage = message().trim();
    if (!userMessage || loading()) return;

    setMessages((prev) => [...prev, { role: "user", content: userMessage }]);
    setMessage("");
    setLoading(true);
    setError(null);

    try {
      const response = await fetch("http://localhost:8080/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message: userMessage,
          history: messages(),
        }),
      });

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      const reader = response.body?.getReader();
      if (!reader) throw new Error("No response body");

      const decoder = new TextDecoder();
      let assistantMessage = "";
      let sseBuffer = "";

      // Add empty assistant message
      setMessages((prev) => [...prev, { role: "assistant", content: "" }]);

      while (true) {
        // eslint-disable-next-line no-await-in-loop -- streaming requires sequential reads
        const { done, value } = await reader.read();
        if (done) break;

        sseBuffer += decoder.decode(value, { stream: true });
        const lines = sseBuffer.split("\n");
        sseBuffer = lines.pop() ?? "";

        for (const line of lines) {
          if (line.startsWith("data: ")) {
            const data = line.slice(6);
            if (data === "[DONE]") continue;

            // Some providers stream cumulative snapshots, while others stream deltas.
            if (data.startsWith(assistantMessage)) {
              assistantMessage = data;
            } else {
              assistantMessage += data;
            }

            setMessages((prev) => {
              const updated = [...prev];
              updated[updated.length - 1] = {
                role: "assistant",
                content: assistantMessage,
              };
              return updated;
            });
          }
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to send message");
    } finally {
      setLoading(false);
    }
  };

  return (
    <main>
      <Title>AI | Luna</Title>
      <header>
        <hgroup>
          <h1>Luna AI</h1>
          <p>Chat with AI powered by Pydantic AI + FastAPI.</p>
        </hgroup>
      </header>

      <section class="chat-container">
        <div class="chat-messages">
          <For each={messages()}>
            {(msg) => (
              <div class={`message message-${msg.role}`}>
                <div class="message-content">{msg.content}</div>
              </div>
            )}
          </For>
          {loading() && (
            <div class="message message-assistant">
              <div class="message-content loading">Thinking...</div>
            </div>
          )}
          {error() && (
            <div class="message message-error">
              <div class="message-content">{error()}</div>
            </div>
          )}
        </div>
      </section>

      <footer>
        <form class="chat-input" onSubmit={handleSubmit}>
          <input
            type="text"
            value={message()}
            onInput={(e) => setMessage(e.currentTarget.value)}
            placeholder="Type your message..."
            disabled={loading()}
          />
          <button type="submit" disabled={loading() || !message().trim()}>
            Send
          </button>
        </form>

        <p class="hint">
          Make sure the API server is running: <code>moon run api:dev</code>
        </p>
      </footer>
    </main>
  );
}
