export class EnemyCatalog {
  #tables;
  #text;
  #records;
  #bindings;
  #runtime;
  #definitions;

  constructor(tables, text, records, bindings, runtime) {
    this.#tables = tables;
    this.#text = text;
    this.#records = records;
    this.#bindings = bindings;
    this.#runtime = runtime;
  }

  extend(catalog) {
    const rows = this.#tables.table('NpcData');
    const covered = new Set(catalog.entries.flatMap((row) => this.#records.ids(row.entry)));

    this.#definitions = {
      skill: new Map(this.#tables.table('Skill').map((row) => [row.ID.Value, row])),
      effect: new Map(this.#tables.table('SkillAbnormal').map((row) => [row.ID.Value, row])),
      npc: new Map(rows.map((row) => [row.ID.Value, row])),
    };

    for (const row of rows) {
      if (covered.has(row.ID.Value) === false && this.eligible(row) === true) {
        catalog.entries.push(this.create(row));
      }
    }

    catalog.entries.forEach((entry) => this.populate(entry));
  }

  eligible(row) {
    const monster = row.NpcType === 'ENpcType::Monster';
    const helper =
      row.NpcType === 'ENpcType::Summon' &&
      row.RelationshipEntity === 'ERelationshipEntity::Monster';

    if (monster === false && helper === false) {
      return false;
    }

    if (/(^test|(?:^|_)test(?:_|\d|$)|sandbag)/i.test(row.Name) === true) {
      return false;
    }

    const id = row.ID.Value;
    const named = this.#text.named('npc', row) || this.#bindings.observed(id);

    return named && (this.#bindings.skills(id).size !== 0 || this.#bindings.effects(id).size !== 0);
  }

  create(row) {
    const key = `enemy:${row.ID.Value}`;
    const entry = {
      key,
      name: this.#text.title('npc', row, 'en'),
      ...(this.#text.named('npc', row) ? {} : { nameKnown: false }),
      icon: null,
      categories: this.helper(row) ? ['mechanic'] : [],
      classes: [],
      forms: [{ key, name: 'Exact variant', matchIds: [row.ID.Value] }],
    };

    this.#text.entry(key, 'npc', row, (language) => this.context(row, language));
    this.#text.form(key, key, 'npc', row, false);

    return { entry, context: this.context(row, 'en'), skills: [], effects: [] };
  }

  helper(row) {
    return (
      row.NpcTargetAble !== 'ENpcTargetAble::Able' || row.NpcNameplate === 'ENpcNameplate::Hidden'
    );
  }

  context(row, language) {
    let kind = language === 'ru' ? 'Противник' : 'Enemy';

    if (this.#runtime.npcs[row.ID.Value].boss === true) {
      kind = language === 'ru' ? 'Босс' : 'Boss';
    }

    if (this.helper(row) === true) {
      kind = language === 'ru' ? 'Источник механики' : 'Mechanic source';
    }

    const level = language === 'ru' ? 'Уровень' : 'Level';

    return `${kind} · ${level} ${row.Level} · ID ${row.ID.Value}`;
  }

  populate(enemy) {
    const ids = this.#records.ids(enemy.entry);
    const npc = this.#definitions.npc.get(ids[0]);

    this.refresh(enemy, npc);
    this.children(enemy, ids, 'skill');
    this.children(enemy, ids, 'effect');
  }

  refresh(enemy, npc) {
    if (this.#text.named('npc', npc) === false) {
      enemy.entry.nameKnown = false;
    }

    this.#text.entry(enemy.entry.key, 'npc', npc, (language) => {
      const previous = this.#text.context(enemy.entry.key, language);
      const label = language === 'ru' ? 'Уровень' : 'Level';

      return previous.replace(new RegExp(`${label} \\d+`), `${label} ${npc.Level}`);
    });

    enemy.entry.name = this.#text.title('npc', npc, 'en');
    enemy.context = this.#text.context(enemy.entry.key, 'en');
  }

  children(enemy, ids, kind) {
    const entries = kind === 'skill' ? enemy.skills : enemy.effects;
    const existing = new Set(entries.flatMap((entry) => this.#records.ids(entry)));

    for (const id of this.#bindings.shared(ids, kind)) {
      const row = this.#definitions[kind].get(id);

      if (existing.has(id) === true || this.visible(row, kind) === false) {
        continue;
      }

      const key = `${enemy.entry.key}:${kind}:${id}`;
      const entry = this.#records.create(key, kind, row, []);

      this.#records.exact(entry, row, kind);
      entries.push(entry);
    }

    entries.forEach((entry) => this.childContext(enemy, entry, kind));
  }

  visible(row, kind) {
    if (kind === 'skill') {
      return true;
    }

    return this.#runtime.effects[row.ID.Value].iconVisible && this.#text.named(kind, row);
  }

  childContext(enemy, entry, kind) {
    const id = this.#records.ids(entry)[0];
    const row = this.#definitions[kind].get(id);

    if (
      this.#text.named(kind, row) === false &&
      ['en', 'ru'].every(
        (language) =>
          this.#text.label(entry.key, language) === this.#text.title(kind, row, language),
      ) === true
    ) {
      entry.nameKnown = false;
    }
    this.#text.compact(entry.key, (language) => {
      const parent = this.#text.label(enemy.entry.key, language);
      const context = this.#text.context(enemy.entry.key, language);

      return `${parent} · ${context} · ID ${id}`;
    });
  }
}
