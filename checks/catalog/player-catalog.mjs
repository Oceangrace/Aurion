export class PlayerCatalog {
  #tables;
  #text;
  #relations;
  #records;
  #runtime;
  #skillFamilies = new Map();
  #effectFamilies = new Map();

  constructor(tables, text, relations, records, runtime) {
    this.#tables = tables;
    this.#text = text;
    this.#relations = relations;
    this.#records = records;
    this.#runtime = runtime;
  }

  extend(catalog) {
    this.skills(catalog.skills);
    this.effects(catalog.effects);
  }

  index(map, family, entry) {
    const entries = map.get(family) ?? new Set();

    entries.add(entry);
    map.set(family, entries);
  }

  unique(map, family, id) {
    const entries = map.get(family);
    const exact = [...(entries ?? [])].find((entry) => this.#records.ids(entry).includes(id));

    if (exact !== undefined) {
      return exact;
    }

    return entries?.size === 1 ? [...entries][0] : null;
  }

  skills(entries) {
    for (const entry of entries) {
      for (const id of this.#records.ids(entry)) {
        this.index(this.#skillFamilies, this.#relations.root(id), entry);
      }
    }

    const rows = this.#tables.table('Skill');
    const definitions = new Map(rows.map((row) => [row.ID.Value, row]));

    for (const row of rows) {
      if (
        this.#relations.player(row.ID.Value) === false ||
        this.#text.named('skill', row) === false
      ) {
        continue;
      }

      this.skill(row, entries, definitions);
    }
  }

  skill(row, entries, definitions) {
    const root = this.#relations.root(row.ID.Value);
    const classes = this.#relations.classes(row.ID.Value);
    let entry = this.unique(this.#skillFamilies, root, row.ID.Value);

    if (entry === null) {
      const ambiguous = (this.#skillFamilies.get(root)?.size ?? 0) > 1;
      const base = ambiguous ? row : (definitions.get(root) ?? row);
      const key = `skill:restored:${ambiguous ? row.ID.Value : root}`;

      entry = this.#records.create(key, 'skill', base, classes, (language) => {
        const metadata = this.#runtime.skills[base.ID.Value];
        const labels = [];

        if (metadata.automatic === true) {
          labels.push(language === 'ru' ? 'Автоматическое умение' : 'Automatic ability');
        }

        if (base.ChainSkillPrevSkillId.Value !== 0) {
          labels.push(language === 'ru' ? 'Этап цепочки' : 'Chain step');
        }

        return [...labels, `ID ${base.ID.Value}`].join(' · ');
      });
      entries.push(entry);
      this.index(this.#skillFamilies, root, entry);
    }

    this.#records.classes(entry, classes);
    this.#records.add(entry, 'skill', row);
  }

  effects(entries) {
    const rows = this.#tables.table('SkillAbnormal');
    const definitions = new Map(rows.map((row) => [row.ID.Value, row]));

    for (const entry of entries) {
      this.indexEffects(entry, definitions);
    }

    for (const row of rows) {
      if (
        this.#runtime.effects[row.ID.Value].iconVisible !== true ||
        this.#text.named('effect', row) === false
      ) {
        continue;
      }

      this.effect(row, entries);
    }

    this.effectContexts(entries, definitions);
  }

  indexEffects(entry, definitions) {
    for (const id of this.#records.ids(entry)) {
      const row = definitions.get(id);

      this.index(this.#effectFamilies, this.#relations.effectFamily(row, this.#text), entry);
      this.#records.classes(entry, this.#relations.effectClasses(id));
    }
  }

  effect(row, entries) {
    const family = this.#relations.effectFamily(row, this.#text);
    const classes = this.#relations.effectClasses(row.ID.Value);
    let entry = this.unique(this.#effectFamilies, family, row.ID.Value);

    if (entry === null) {
      const key = `effect:restored:${row.ID.Value}`;

      entry = this.#records.create(key, 'effect', row, classes);
      entries.push(entry);
      this.index(this.#effectFamilies, family, entry);
    }

    this.#records.classes(entry, classes);
    this.#records.add(entry, 'effect', row);
  }

  effectContexts(entries, definitions) {
    const skills = new Map(this.#tables.table('Skill').map((row) => [row.ID.Value, row]));

    for (const entry of entries.filter((entry) => entry.key.startsWith('effect:restored:'))) {
      const ids = this.#records.ids(entry);
      const row = definitions.get(ids[0]);
      const sources = new Set(
        this.#relations.sources(row.ID.Value).map((id) => this.#relations.root(id)),
      );

      this.#text.entry(entry.key, 'effect', row, (language) => {
        const title = this.#text.title('effect', row, language);
        const names = [...sources]
          .map((id) => this.#text.title('skill', skills.get(id), language))
          .filter((name) => name !== title);
        const suffix =
          ids.length === 1
            ? `ID ${ids[0]}`
            : `${ids.length} ${language === 'ru' ? 'вариантов' : 'forms'}`;

        return [...new Set(names), suffix].join(' · ');
      });
    }
  }
}
