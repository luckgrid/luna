/**
 * Popover `:popover-open` uses the top layer and cannot sit in the collection body
 * grid — CSS `position: static` does not reinsert it. At ≥lg, close the panel and
 * remove `popover` so the dialog stays in-flow (sticky aside matches right TOC).
 * Below lg, restore `popover="auto"` for mobile off-canvas.
 */
(function () {
  if (!("hidePopover" in HTMLElement.prototype)) return;

  const lg = window.matchMedia("(min-width: 64rem)");

  function syncPanelPopovers() {
    const panels = document.querySelectorAll("dialog[data-panel]");

    for (const panel of panels) {
      if (lg.matches) {
        if (panel.matches(":popover-open")) {
          panel.hidePopover();
        }
        panel.removeAttribute("popover");
      } else if (!panel.hasAttribute("popover")) {
        panel.setAttribute("popover", "auto");
      }
    }
  }

  lg.addEventListener("change", syncPanelPopovers);
  window.addEventListener("resize", syncPanelPopovers);
  syncPanelPopovers();
})();
