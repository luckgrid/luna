/** Catalog search + category filter with client-side re-render (`[data-catalog]`). */
(function () {
  const { normalizeQuery, matchesSearch, escapeHTML } = window.LunaFilter;

  function sortMatches(matches, category) {
    if (!category) return matches;
    return matches.slice().toSorted(function (a, b) {
      if (category === "post-collection") {
        return (a.weight || 0) - (b.weight || 0);
      }
      const aSection = a.kind === "section" ? 0 : 1;
      const bSection = b.kind === "section" ? 0 : 1;
      if (aSection !== bSection) return aSection - bSection;
      return (b.sortDate || 0) - (a.sortDate || 0);
    });
  }

  function renderEntry(entry) {
    return (
      '<li data-search="' +
      escapeHTML(entry.search) +
      '" data-category="' +
      escapeHTML(entry.category || "") +
      '">' +
      entry.card +
      "</li>"
    );
  }

  function initCatalog(root) {
    const form = root.querySelector("[data-catalog-search]");
    const input = root.querySelector("[data-catalog-input]");
    const categoryFilter = root.querySelector("[data-catalog-category]");
    const list = root.querySelector("[data-catalog-list]");
    const status = root.querySelector("[data-catalog-status]");
    const pager = root.querySelector("[data-catalog-pagination]");
    const data = root.querySelector("[data-catalog-entries]");
    if (!input || !list || !data) return;

    const entries = JSON.parse(data.textContent || "[]");
    const initial = list.innerHTML;

    function filter() {
      const q = normalizeQuery(input.value);
      const category = categoryFilter ? (categoryFilter.value || "").trim() : "";

      if (!q && !category) {
        list.innerHTML = initial;
        if (pager) pager.hidden = false;
        if (status) status.hidden = true;
        return;
      }

      let matches = entries.filter(function (entry) {
        return matchesSearch(q, entry.search) && (!category || entry.category === category);
      });

      matches = sortMatches(matches, category);
      list.innerHTML = matches.map(renderEntry).join("");

      if (pager) pager.hidden = true;
      if (status) status.hidden = matches.length !== 0;
    }

    if (form) {
      form.addEventListener("submit", function (event) {
        event.preventDefault();
        filter();
      });
    }

    input.addEventListener("input", filter);
    input.addEventListener("search", filter);
    if (categoryFilter) categoryFilter.addEventListener("change", filter);
  }

  function initAll() {
    for (const root of document.querySelectorAll("[data-catalog]")) {
      initCatalog(root);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initAll);
  } else {
    initAll();
  }
})();
