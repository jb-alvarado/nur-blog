const locale = document.documentElement.lang || navigator.language;
const dateTimeFormatter = new Intl.DateTimeFormat(locale, {
  year: "numeric",
  month: "long",
  day: "numeric",
  hour: "2-digit",
  minute: "2-digit",
  timeZoneName: "short",
});

for (const element of document.querySelectorAll("time[data-local-datetime]")) {
  const date = new Date(element.dateTime);
  if (!Number.isNaN(date.valueOf()))
    element.textContent = dateTimeFormatter.format(date);
}

const menuToggle = document.querySelector("[data-menu-toggle]");
const mobileMenu = document.querySelector("[data-mobile-menu]");

const closeMenu = () => {
  if (!menuToggle || !mobileMenu) return;
  mobileMenu.hidden = true;
  menuToggle.setAttribute("aria-expanded", "false");
};

if (menuToggle && mobileMenu) {
  menuToggle.addEventListener("click", () => {
    const open = mobileMenu.hidden;
    mobileMenu.hidden = !open;
    menuToggle.setAttribute("aria-expanded", String(open));
  });
}

const dialog = document.querySelector("#search-dialog");

if (dialog) {
  const form = dialog.querySelector(".search-modal-form");
  const input = form.elements.q;
  const results = dialog.querySelector(".search-modal-results");
  let request;
  let debounce;

  const showState = (message) => {
    const state = document.createElement("p");
    state.className = "search-state";
    state.textContent = message;
    results.replaceChildren(state);
  };

  const open = () => {
    if (!dialog.open) dialog.showModal();
    requestAnimationFrame(() => input.focus());
  };

  const close = () => dialog.close();

  const search = async () => {
    const query = input.value.trim();
    request?.abort();
    if (query.length < 2) {
      showState(dialog.dataset.searchMinimum);
      return;
    }

    request = new AbortController();
    showState(dialog.dataset.searchSearching);
    const url = new URL(form.action, window.location.origin);
    url.searchParams.set("q", query);
    url.searchParams.set("locale", form.elements.locale.value);
    url.searchParams.set("fragment", "1");

    try {
      const response = await fetch(url, {
        headers: { accept: "text/html" },
        signal: request.signal,
      });
      if (!response.ok) throw new Error("Search failed");
      results.innerHTML = await response.text();
    } catch (error) {
      if (error.name !== "AbortError")
        showState(dialog.dataset.searchUnavailable);
    }
  };

  for (const trigger of document.querySelectorAll("[data-search-open]"))
    trigger.addEventListener("click", (event) => {
      event.preventDefault();
      closeMenu();
      open();
    });

  dialog.querySelector("[data-search-close]").addEventListener("click", close);
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) close();
  });
  dialog.addEventListener("close", () => request?.abort());
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    clearTimeout(debounce);
    search();
  });
  input.addEventListener("input", () => {
    clearTimeout(debounce);
    debounce = setTimeout(search, 180);
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") closeMenu();
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      dialog.open ? close() : open();
    }
  });
}
