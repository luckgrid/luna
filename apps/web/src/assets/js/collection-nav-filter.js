/** Filter-as-you-type for collection sidebar nav lists (`[data-collection-nav]`). */
(function () {
  const { filterListItems, bindSearchInput } = window.LunaFilter;

  function initNav(nav) {
    const input = nav.querySelector("[data-collection-nav-input]");
    const list = nav.querySelector("[data-collection-nav-list]");
    const status = nav.querySelector("[data-collection-nav-status]");
    if (!input || !list) return;

    function filter() {
      filterListItems(list, input.value, status);
    }

    bindSearchInput(input, filter);
  }

  function initAll() {
    for (const nav of document.querySelectorAll("[data-collection-nav]")) {
      initNav(nav);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initAll);
  } else {
    initAll();
  }
})();
