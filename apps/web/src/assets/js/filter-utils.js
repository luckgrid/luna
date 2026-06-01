/** Shared list/search filter helpers for Hugo static pages. */
(function (global) {
  function normalizeQuery(value) {
    return (value || "").toLowerCase().trim();
  }

  function matchesSearch(query, haystack) {
    return !query || (haystack || "").includes(query);
  }

  function escapeHTML(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  /** Toggle `hidden` on `<li data-search>` items; update optional status element. */
  function filterListItems(list, query, statusEl) {
    const q = normalizeQuery(query);
    const items = list.querySelectorAll("li");
    let visible = 0;

    for (const item of items) {
      const hay = item.getAttribute("data-search") || "";
      const show = matchesSearch(q, hay);
      item.hidden = !show;
      if (show) visible++;
    }

    if (statusEl) statusEl.hidden = visible !== 0;
    return visible;
  }

  function bindSearchInput(input, onFilter) {
    input.addEventListener("input", onFilter);
    input.addEventListener("search", onFilter);
  }

  global.LunaFilter = {
    normalizeQuery,
    matchesSearch,
    escapeHTML,
    filterListItems,
    bindSearchInput,
  };
})(typeof window !== "undefined" ? window : globalThis);
