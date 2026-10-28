export class CatalogRecords {
  #text;
  #runtime;

  constructor(text, runtime) {
    this.#text = text;
    this.#runtime = runtime;
  }

  ids(entry) {
    return [...new Set(entry.forms.flatMap((form) => form.matchIds))].sort((a, b) => a - b);
  }

  create(key, kind, row, classes, context = null) {
    const metadata = kind === 'skill' ? this.#runtime.skills : this.#runtime.effects;
    const categories = kind === 'skill' ? ['ability'] : [metadata[row.ID.Value].disposition];
    const entry = {
      key,
      name: this.#text.title(kind, row, 'en'),
      ...(this.#text.named(kind, row) ? {} : { nameKnown: false }),
      icon: metadata[row.ID.Value].icon ?? null,
      categories,
      classes: [...classes].sort(),
      forms: [],
    };

    this.#text.entry(key, kind, row, context);

    return entry;
  }

  add(entry, kind, row) {
    if (this.ids(entry).includes(row.ID.Value) === true) {
      return;
    }

    const key = `${kind}-id:${row.ID.Value}`;

    entry.forms.push({ key, name: `ID ${row.ID.Value}`, matchIds: [row.ID.Value] });
    this.#text.form(entry.key, key, kind, row, entry.forms.length > 1);
    this.refreshUnion(entry);
  }

  exact(entry, row, kind) {
    entry.forms.push({ key: entry.key, name: 'Exact variant', matchIds: [row.ID.Value] });
    this.#text.form(entry.key, entry.key, kind, row, false);
  }

  refreshUnion(entry) {
    const union = entry.forms.find((form) => form.key.endsWith(':any-supported'));

    if (union !== undefined) {
      union.matchIds = this.ids(entry);
    }
  }

  classes(entry, classes) {
    entry.classes = [...new Set([...entry.classes, ...classes])].sort();
  }
}
