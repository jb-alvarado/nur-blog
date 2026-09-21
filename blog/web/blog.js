const dialog = document.querySelector("#search-dialog");

if (dialog) {
  const form = dialog.querySelector(".search-modal-form");
  const input = form.elements.q;
  const results = dialog.querySelector(".search-modal-results");
  let request;
  let debounce;

  const open = () => {
    if (!dialog.open) dialog.showModal();
    requestAnimationFrame(() => input.focus());
  };

  const close = () => dialog.close();

  const search = async () => {
    const query = input.value.trim();
    request?.abort();
    if (query.length < 2) {
      results.innerHTML =
        '<p class="search-state">Enter at least two characters.</p>';
      return;
    }

    request = new AbortController();
    results.innerHTML = '<p class="search-state">Searching…</p>';
    const url = new URL(form.action, window.location.origin);
    url.searchParams.set("q", query);
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
        results.innerHTML =
          '<p class="search-state">Search is currently unavailable.</p>';
    }
  };

  for (const trigger of document.querySelectorAll("[data-search-open]"))
    trigger.addEventListener("click", (event) => {
      event.preventDefault();
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
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      dialog.open ? close() : open();
    }
  });
}
