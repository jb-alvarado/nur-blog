const TRANSLATIONS = __NUR_BLOG_TRANSLATIONS__;

class NurCmsBlogSettings extends HTMLElement {
  connectedCallback() {
    if (this.initialized) return;
    this.initialized = true;
    this.unsubscribeLocale = this.context.onLocaleChange(() => this.renderAndFill());
    this.render();
    this.load();
  }

  disconnectedCallback() {
    this.unsubscribeLocale?.();
  }

  t(key) {
    const requested = (this.context.locale() || "en").replaceAll("_", "-").toLowerCase();
    const exact = Object.keys(TRANSLATIONS).find((locale) => locale.toLowerCase() === requested);
    const base = Object.keys(TRANSLATIONS).find(
      (locale) => locale.toLowerCase() === requested.split("-")[0],
    );
    return TRANSLATIONS[exact]?.[key] ?? TRANSLATIONS[base]?.[key] ?? TRANSLATIONS.en[key] ?? key;
  }

  async load() {
    try {
      const response = await this.context.request("/settings");
      if (!response.ok) throw new Error(this.t("load_error"));
      this.settings = await response.json();
      this.editingLocale = this.settings.default_locale;
      this.fill();
    } catch (error) {
      this.status(error.message, "error");
    }
  }

  renderAndFill() {
    if (this.settings) this.captureLocalization();
    this.render();
    if (this.settings) this.fill();
  }

  render() {
    this.replaceChildren();
    const form = document.createElement("form");
    form.className = "nur-blog-settings";
    form.addEventListener("submit", (event) => this.save(event));
    form.append(
      this.defaultLocaleField(),
      this.multilingualField(),
      this.faviconField(),
      this.field(this.t("article_type"), "article_type", "text"),
      this.field(this.t("page_type"), "page_type", "text"),
      this.field(this.t("posts_per_page"), "posts_per_page", "number"),
      this.localizationEditor(),
    );
    this.message = document.createElement("p");
    this.message.className = "nur-blog-settings__message";
    const submit = document.createElement("button");
    submit.type = "submit";
    submit.textContent = this.t("save");
    form.append(this.message, submit);
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

  defaultLocaleField() {
    const label = document.createElement("label");
    label.textContent = this.t("default_locale");
    const select = document.createElement("select");
    select.name = "default_locale";
    select.addEventListener("change", () => {
      if (this.settings) this.settings.default_locale = select.value;
    });
    label.append(select);
    return label;
  }

  multilingualField() {
    const label = document.createElement("label");
    label.className = "nur-blog-settings__toggle";
    const input = document.createElement("input");
    input.name = "multilingual_enabled";
    input.type = "checkbox";
    const text = document.createElement("span");
    text.append(this.t("multilingual_enabled"));
    const hint = document.createElement("small");
    hint.textContent = this.t("multilingual_hint");
    text.append(hint);
    label.append(input, text);
    return label;
  }

  localizationEditor() {
    const section = document.createElement("fieldset");
    section.className = "nur-blog-settings__localizations";
    const legend = document.createElement("legend");
    legend.textContent = this.t("localizations");
    const toolbar = document.createElement("div");
    toolbar.className = "nur-blog-settings__locale-toolbar";
    this.localeSelect = document.createElement("select");
    this.localeSelect.setAttribute("aria-label", this.t("locale"));
    this.localeSelect.addEventListener("change", () => this.changeLocale(this.localeSelect.value));
    this.newLocale = document.createElement("input");
    this.newLocale.type = "text";
    this.newLocale.placeholder = this.t("locale_placeholder");
    this.newLocale.setAttribute("aria-label", this.t("locale"));
    const addLocale = document.createElement("button");
    addLocale.type = "button";
    addLocale.textContent = this.t("add_locale");
    addLocale.addEventListener("click", () => this.addLocale());
    const removeLocale = document.createElement("button");
    removeLocale.type = "button";
    removeLocale.textContent = this.t("remove_locale");
    removeLocale.addEventListener("click", () => this.removeLocale());
    toolbar.append(this.localeSelect, this.newLocale, addLocale, removeLocale);
    this.localizedFields = document.createElement("div");
    this.localizedFields.className = "nur-blog-settings__localized-fields";
    this.localizedFields.append(
      this.field(this.t("site_name"), "site_name", "text"),
      this.field(this.t("site_description"), "site_description", "text"),
      this.field(this.t("index_page"), "index_page_slug", "text"),
    );
    const navigation = document.createElement("fieldset");
    navigation.className = "nur-blog-settings__navigation";
    const navigationLegend = document.createElement("legend");
    navigationLegend.textContent = this.t("navigation");
    this.navigationRows = document.createElement("div");
    this.navigationRows.className = "nur-blog-settings__rows";
    const addLink = document.createElement("button");
    addLink.type = "button";
    addLink.textContent = this.t("add_link");
    addLink.addEventListener("click", () => this.addNavigationRow());
    navigation.append(navigationLegend, this.navigationRows, addLink);
    this.localizedFields.append(navigation);
    section.append(legend, toolbar, this.localizedFields);
    return section;
  }

  faviconField() {
    const field = document.createElement("div");
    field.className = "nur-blog-settings__favicon";
    const label = document.createElement("label");
    label.htmlFor = "nur-blog-favicon-url";
    label.textContent = this.t("favicon_url");
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
      select.textContent = this.t("select_media");
      select.addEventListener("click", () => this.selectFavicon());
      controls.append(select);
    }
    const clear = document.createElement("button");
    clear.type = "button";
    clear.textContent = this.t("clear");
    clear.addEventListener("click", () => {
      input.value = "";
      this.updateFaviconPreview();
    });
    controls.append(clear);
    this.faviconPreview = document.createElement("img");
    this.faviconPreview.className = "nur-blog-settings__favicon-preview";
    this.faviconPreview.alt = this.t("favicon_preview");
    field.append(label, controls, this.faviconPreview);
    return field;
  }

  async selectFavicon() {
    try {
      const media = await this.context.selectMedia({ types: ["image"] });
      if (!media) return;
      this.querySelector('[name="favicon_url"]').value = media.url;
      this.updateFaviconPreview();
    } catch (error) {
      this.status(error.message || this.t("media_error"), "error");
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
    form.elements.default_locale.replaceChildren(
      ...this.settings.localizations.map((item) => new Option(item.locale, item.locale)),
    );
    form.elements.multilingual_enabled.checked = this.settings.multilingual_enabled;
    for (const name of ["default_locale", "favicon_url", "article_type", "page_type", "posts_per_page"])
      form.elements[name].value = this.settings[name] ?? "";
    if (!this.settings.localizations.some((item) => item.locale === this.editingLocale))
      this.editingLocale = this.settings.default_locale;
    this.localeSelect.replaceChildren(
      ...this.settings.localizations.map((item) => new Option(item.locale, item.locale)),
    );
    this.localeSelect.value = this.editingLocale;
    this.fillLocalization();
    this.updateFaviconPreview();
  }

  fillLocalization() {
    const localization = this.currentLocalization();
    if (!localization) return;
    const form = this.querySelector("form");
    for (const name of ["site_name", "site_description", "index_page_slug"])
      form.elements[name].value = localization[name] ?? "";
    this.navigationRows.replaceChildren();
    for (const item of localization.navigation ?? []) this.addNavigationRow(item);
  }

  captureLocalization() {
    const localization = this.currentLocalization();
    const form = this.querySelector("form");
    if (!localization || !form) return;
    localization.site_name = form.elements.site_name.value.trim();
    localization.site_description = form.elements.site_description.value.trim();
    localization.index_page_slug = form.elements.index_page_slug.value.trim() || null;
    localization.navigation = [...this.navigationRows.children].map((row) => ({
      label: row.querySelector("[data-navigation-label]").value.trim(),
      href: row.querySelector("[data-navigation-href]").value.trim(),
    }));
  }

  currentLocalization() {
    return this.settings?.localizations.find((item) => item.locale === this.editingLocale);
  }

  changeLocale(locale) {
    this.captureLocalization();
    this.editingLocale = locale;
    this.fillLocalization();
  }

  addLocale() {
    this.captureLocalization();
    const locale = this.newLocale.value.trim().replaceAll("_", "-");
    if (!/^[A-Za-z]{2,8}(?:-[A-Za-z0-9]{1,8})*$/.test(locale)) {
      this.status(this.t("invalid_locale"), "error");
      return;
    }
    if (this.settings.localizations.some((item) => item.locale.toLowerCase() === locale.toLowerCase())) {
      this.status(this.t("duplicate_locale"), "error");
      return;
    }
    this.settings.localizations.push({ locale, site_name: "", site_description: "", index_page_slug: null, navigation: [] });
    this.editingLocale = locale;
    this.newLocale.value = "";
    this.fill();
    this.status("", "");
  }

  removeLocale() {
    const defaultLocale = this.querySelector('[name="default_locale"]').value.trim();
    if (this.editingLocale.toLowerCase() === defaultLocale.toLowerCase()) {
      this.status(this.t("remove_default_locale"), "error");
      return;
    }
    this.settings.localizations = this.settings.localizations.filter((item) => item.locale !== this.editingLocale);
    this.editingLocale = this.settings.localizations[0]?.locale;
    this.fill();
  }

  addNavigationRow(item = { label: "", href: "" }) {
    const row = document.createElement("div");
    row.className = "nur-blog-settings__row";
    const label = document.createElement("input");
    label.type = "text";
    label.placeholder = this.t("link_label");
    label.value = item.label;
    label.dataset.navigationLabel = "";
    const href = document.createElement("input");
    href.type = "text";
    href.placeholder = this.t("link_target");
    href.value = item.href;
    href.dataset.navigationHref = "";
    const remove = document.createElement("button");
    remove.type = "button";
    remove.textContent = this.t("remove");
    remove.addEventListener("click", () => row.remove());
    row.append(label, href, remove);
    this.navigationRows.append(row);
  }

  async save(event) {
    event.preventDefault();
    this.captureLocalization();
    const form = event.currentTarget;
    const value = (name) => form.elements[name].value.trim();
    const settings = {
      default_locale: value("default_locale").replaceAll("_", "-"),
      multilingual_enabled: form.elements.multilingual_enabled.checked,
      favicon_url: value("favicon_url") || null,
      article_type: value("article_type"),
      page_type: value("page_type"),
      posts_per_page: Number(form.elements.posts_per_page.value),
      localizations: this.settings.localizations,
    };
    try {
      const response = await this.context.request("/settings", {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(settings),
      });
      if (!response.ok) throw new Error(this.t("save_error"));
      this.settings = structuredClone(settings);
      this.status(this.t("saved"), "success");
      this.context.notify("success", this.t("saved_notification"));
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
