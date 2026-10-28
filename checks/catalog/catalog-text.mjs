export class CatalogText {
  #strings;
  #locales = new Map();
  #skillStrings;
  #effectStrings;
  #translations;

  constructor(translations) {
    this.#translations = translations;
  }

  load(tables) {
    this.#strings = new Map(tables.table('String').map((row) => [row.Name, row.Body]));
    this.#skillStrings = new Map(tables.table('SkillString').map((row) => [row.Name, row]));
    this.#effectStrings = new Map(
      tables.table('SkillAbnormalString').map((row) => [row.Name, row]),
    );
    this.#locales.set('en', tables.locale('en-US'));
    this.#locales.set('ru', tables.locale('ru-RU'));
  }

  resolve(reference, language) {
    const key = reference?.Key !== 'None' ? reference?.Key : reference?.StringKey;
    const value = this.#strings.get(key) ?? reference;
    const resolved = value?.Key !== 'None' ? value?.Key : value?.StringKey;

    return this.#locales.get(language)[resolved] ?? null;
  }

  name(kind, row, language) {
    if (kind === 'npc') {
      return this.resolve(row.Desc, language);
    }

    const definition = this.definition(kind, row);
    const field = kind === 'skill' ? 'SkillName' : 'DescName';

    return this.resolve(definition?.[field], language);
  }

  definition(kind, row) {
    if (kind === 'skill') {
      return this.#skillStrings.get(row.SkillString_Key);
    }

    return this.#effectStrings.get(row.SkillAbnormalString_Key);
  }

  readable(value) {
    if (typeof value !== 'string' || value.trim() === '') {
      return false;
    }

    return (
      /\p{L}/u.test(value) &&
      /_|\btest\b|\bdebug\b|\bdummy\b|для проверки|служебн|образец|<[^>]*>|\{[^}]*\}/iu.test(
        value,
      ) === false
    );
  }

  named(kind, row) {
    return ['en', 'ru'].some((language) => this.readable(this.name(kind, row, language)));
  }

  title(kind, row, language) {
    const value = this.name(kind, row, language);

    if (this.readable(value) === true) {
      return value;
    }

    const fallback = this.name(kind, row, 'en');

    if (this.readable(fallback) === true) {
      return fallback;
    }

    const labels = {
      skill: ['Unnamed ability', 'Умение без названия'],
      effect: ['Unnamed effect', 'Эффект без названия'],
      npc: ['Unnamed source', 'Источник без названия'],
    };

    return labels[kind][language === 'ru' ? 1 : 0];
  }

  entry(key, kind, row, context = null) {
    for (const language of ['en', 'ru']) {
      const existing = this.#translations[language].entries[key] ?? {};

      this.#translations[language].entries[key] = {
        ...existing,
        name: this.title(kind, row, language),
        ...(context === null ? {} : { context: context(language) }),
        forms: existing.forms ?? {},
      };
    }
  }

  labelEntry(key, kind, row) {
    for (const language of ['en', 'ru']) {
      const previous = this.#translations[language].entries[key] ?? {};

      this.#translations[language].entries[key] = {
        ...previous,
        name: this.title(kind, row, language),
      };
    }
  }

  form(entryKey, key, kind, row, variant) {
    if (entryKey === key && variant === false) {
      return;
    }

    for (const language of ['en', 'ru']) {
      const entry = this.#translations[language].entries[entryKey];
      const forms = entry.forms ?? {};

      if (forms[key] !== undefined) {
        continue;
      }

      const title = this.title(kind, row, language);
      const label = language === 'ru' ? 'Вариант' : 'Variant';

      forms[key] = { name: variant ? `${label} · ID ${row.ID.Value}` : title };
      entry.forms = forms;
    }
  }

  context(key, language) {
    return this.#translations[language].entries[key]?.context ?? '';
  }

  compact(key, context) {
    for (const language of ['en', 'ru']) {
      const row = this.#translations[language].entries[key];

      if (row.context === context(language)) {
        delete row.context;
      }

      if (row.forms?.[key]?.name === row.name) {
        delete row.forms[key];
      }

      if (Object.keys(row.forms ?? {}).length === 0) {
        delete row.forms;
      }
    }
  }

  label(key, language) {
    return this.#translations[language].entries[key]?.name ?? null;
  }
}
