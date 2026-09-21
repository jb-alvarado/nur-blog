class NurCmsBlogSettings extends HTMLElement {
  connectedCallback() {
    if (this.initialized) return;
    this.initialized = true;
    this.render();
    this.load();
  }

  async load() {
    try {
      const response = await this.context.request("/settings");
      if (!response.ok) throw new Error("Could not load blog settings.");
      this.settings = await response.json();
      this.fill();
    } catch (error) {
      this.status(error.message, "error");
    }
  }

  render() {
    this.replaceChildren();
    const form = document.createElement("form");
    form.className = "nur-blog-settings";
    form.addEventListener("submit", (event) => this.save(event));

    form.append(
      this.field("Site name", "site_name", "text"),
      this.field("Site description", "site_description", "text"),
      this.faviconField(),
      this.field("Article content type slug", "article_type", "text"),
      this.field("Page content type slug", "page_type", "text"),
      this.field("Index page slug", "index_page_slug", "text"),
      this.field("Articles per page", "posts_per_page", "number"),
    );

    const navigation = document.createElement("fieldset");
    navigation.className = "nur-blog-settings__navigation";
    const legend = document.createElement("legend");
    legend.textContent = "Navigation";
    this.navigationRows = document.createElement("div");
    this.navigationRows.className = "nur-blog-settings__rows";
    const add = document.createElement("button");
    add.type = "button";
    add.textContent = "Add link";
    add.addEventListener("click", () => this.addNavigationRow());
    navigation.append(legend, this.navigationRows, add);

    this.message = document.createElement("p");
    this.message.className = "nur-blog-settings__message";
    const submit = document.createElement("button");
    submit.type = "submit";
    submit.textContent = "Save settings";
    form.append(navigation, this.message, submit);
    this.append(form);
  }

  field(labelText, name, type, placeholder = "") {
    const label = document.createElement("label");
    label.textContent = labelText;
    const input = document.createElement("input");
    input.name = name;
    input.type = type;
    input.placeholder = placeholder;
    if (type === "number") {
      input.min = "1";
      input.max = "24";
    }
    label.append(input);
    return label;
  }

  faviconField() {
    const field = document.createElement("div");
    field.className = "nur-blog-settings__favicon";
    const label = document.createElement("label");
    label.htmlFor = "nur-blog-favicon-url";
    label.textContent = "Favicon URL";
    const controls = document.createElement("div");
    controls.className = "nur-blog-settings__favicon-controls";
    const input = document.createElement("input");
    input.id = "nur-blog-favicon-url";
    input.name = "favicon_url";
    input.type = "text";
    input.placeholder = "/uploads/favicon.svg";
    input.addEventListener("input", () => this.updateFaviconPreview());
    controls.append(input);

    if (typeof this.context?.selectMedia === "function") {
      const select = document.createElement("button");
      select.type = "button";
      select.textContent = "Select from media";
      select.addEventListener("click", () => this.selectFavicon());
      controls.append(select);
    }

    const clear = document.createElement("button");
    clear.type = "button";
    clear.textContent = "Clear";
    clear.addEventListener("click", () => {
      input.value = "";
      this.updateFaviconPreview();
    });
    controls.append(clear);

    this.faviconPreview = document.createElement("img");
    this.faviconPreview.className = "nur-blog-settings__favicon-preview";
    this.faviconPreview.alt = "Favicon preview";
    field.append(label, controls, this.faviconPreview);
    return field;
  }

  async selectFavicon() {
    try {
      const media = await this.context.selectMedia({ types: ["image"] });
      if (!media) return;
      const input = this.querySelector('[name="favicon_url"]');
      input.value = media.url;
      this.updateFaviconPreview();
    } catch (error) {
      this.status(error.message || "Could not select favicon.", "error");
    }
  }

  updateFaviconPreview() {
    const url = this.querySelector('[name="favicon_url"]')?.value.trim();
    this.faviconPreview.hidden = !url;
    if (url) this.faviconPreview.src = url;
    else this.faviconPreview.removeAttribute("src");
  }

  fill() {
    const form = this.querySelector("form");
    for (const name of [
      "site_name",
      "site_description",
      "favicon_url",
      "article_type",
      "page_type",
      "index_page_slug",
      "posts_per_page",
    ]) {
      form.elements[name].value = this.settings[name] ?? "";
    }
    this.navigationRows.replaceChildren();
    for (const item of this.settings.navigation ?? [])
      this.addNavigationRow(item);
    this.updateFaviconPreview();
  }

  addNavigationRow(item = { label: "", href: "" }) {
    const row = document.createElement("div");
    row.className = "nur-blog-settings__row";
    const label = document.createElement("input");
    label.type = "text";
    label.placeholder = "Label";
    label.value = item.label;
    label.dataset.navigationLabel = "";
    const href = document.createElement("input");
    href.type = "text";
    href.placeholder = "/about or https://example.com";
    href.value = item.href;
    href.dataset.navigationHref = "";
    const remove = document.createElement("button");
    remove.type = "button";
    remove.textContent = "Remove";
    remove.addEventListener("click", () => row.remove());
    row.append(label, href, remove);
    this.navigationRows.append(row);
  }

  async save(event) {
    event.preventDefault();
    const form = event.currentTarget;
    const value = (name) => form.elements[name].value.trim();
    const navigation = [...this.navigationRows.children].map((row) => ({
      label: row.querySelector("[data-navigation-label]").value.trim(),
      href: row.querySelector("[data-navigation-href]").value.trim(),
    }));
    const settings = {
      site_name: value("site_name"),
      site_description: value("site_description"),
      favicon_url: value("favicon_url") || null,
      article_type: value("article_type"),
      page_type: value("page_type"),
      index_page_slug: value("index_page_slug") || null,
      posts_per_page: Number(form.elements.posts_per_page.value),
      navigation,
    };

    try {
      const response = await this.context.request("/settings", {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(settings),
      });
      if (!response.ok) throw new Error("Could not save blog settings.");
      this.settings = settings;
      this.status("Saved.", "success");
      this.context.notify("success", "Blog settings saved.");
    } catch (error) {
      this.status(error.message, "error");
    }
  }

  status(text, kind) {
    this.message.textContent = text;
    this.message.dataset.kind = kind;
  }
}

customElements.define("nur-cms-blog-settings", NurCmsBlogSettings);
