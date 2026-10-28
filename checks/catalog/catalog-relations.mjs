export class CatalogRelations {
  #tables;
  #runtime;
  #text;
  #classes = new Map();
  #children = new Map();
  #parents = new Map();
  #sources = new Map();
  #common = new Set();
  #public;
  #aliases = new Map();
  #supported = new Set([
    'Gladiator',
    'Templar',
    'Assassin',
    'Ranger',
    'Sorcerer',
    'Elementalist',
    'Cleric',
    'Chanter',
  ]);

  constructor(tables, runtime, playerIds, text) {
    this.#tables = tables;
    this.#runtime = runtime;
    this.#public = new Set(playerIds);
    this.#text = text;
  }

  load() {
    this.seed();
    this.specializations();
    this.chains();
    this.effects();
    this.procs();
    this.propagate();
    this.alternatives();
    this.propagate();
    this.families();
    this.propagate();
  }

  seed() {
    for (const [id, metadata] of Object.entries(this.#runtime.skills)) {
      this.addClass(Number(id), metadata.className);
    }

    for (const row of this.#tables.table('SkillAcquireData')) {
      this.addClass(row.SkillId.Value, row.ClassType.split('::')[1]);

      if (this.#supported.has(row.ClassType.split('::')[1]) === true) {
        this.#public.add(row.SkillId.Value);
      }
    }

    for (const row of this.#tables.table('SkillList')) {
      this.seedList(row);
    }

    for (const row of this.#tables.table('Item')) {
      if (row.UseEffect === 'EItemUseEffect::Skill') {
        this.#common.add(Number(row.UseEffectValues[0]));
      }
    }
  }

  seedList(row) {
    const className = row.ClassType.split('::')[1];

    for (const id of row.SkillIdList) {
      this.addClass(id, className);

      if (this.#supported.has(className) === true) {
        this.#public.add(id);
      }

      if (className === 'None') {
        this.#common.add(id);
      }
    }
  }

  specializations() {
    for (const row of this.#tables.table('SpecializedSkill')) {
      const parent = row.ChainParentSkillId3.Value;
      const child = row.ChainChildSkillId3.Value;

      if (row.ChainChildSkillId2.Value !== child) {
        throw new Error('Unsupported specialization relation');
      }

      this.link(parent, child);

      if (parent !== child) {
        const parents = this.#parents.get(child) ?? new Set();

        parents.add(parent);
        this.#parents.set(child, parents);
      }
    }

    for (const row of this.#tables.table('SpecializedSkillParts')) {
      const className = row.ClassType.split('::')[1];

      this.addClass(row.ParentSkillId.Value, className);
      this.addClass(row.ID.Value, className);
    }
  }

  chains() {
    const charges = new Map(this.#tables.table('SkillCharge').map((row) => [row.ID.Value, row]));

    for (const row of this.#tables.table('Skill')) {
      this.link(row.ChainSkillPrevSkillId.Value, row.ID.Value);

      for (const charge of charges.get(row.ChargeId.Value)?.ChargeSkillDataList ?? []) {
        this.link(row.ID.Value, charge.ChargeSkillId.Value);
      }
    }
  }

  effects() {
    const owners = new Map();
    const levels = new Map();

    for (const row of this.#tables.table('Skill')) {
      for (const reference of row.SkillEffectTimeDataList) {
        this.append(owners, reference.SkillEffectGroupId.Value, row.ID.Value);
      }
    }

    for (const row of this.#tables.table('SkillEffectLv')) {
      const values = levels.get(row.SkillEffectLvGroupId) ?? [];

      values.push(row.EffectValueList);
      levels.set(row.SkillEffectLvGroupId, values);
    }

    for (const row of this.#tables.table('SkillEffect')) {
      this.effectSources(row, owners.get(row.SkillEffectGroupId.Value) ?? [], levels);
    }
  }

  effectSources(row, owners, levels) {
    const positions = {
      'ESkillEffectType::Abnormal': 2,
      'ESkillEffectType::Abnormal_True': 2,
      'ESkillEffectType::Abnormal_Passive': 0,
      'ESkillEffectType::Abnormal_Cc': 1,
    };
    const position = positions[row.EffectType];

    if (position === undefined) {
      return;
    }

    const variants = [row.EffectValueList, ...(levels.get(row.SkillEffectLvGroupId) ?? [])];

    for (const values of variants) {
      const id = Number(values[position]);

      if (this.#runtime.effects[id] !== undefined) {
        owners.forEach((owner) => this.append(this.#sources, id, owner));
      }
    }
  }

  procs() {
    for (const row of this.#tables.table('SkillAbnormalEffect')) {
      if (row.AbnormalEffectType !== 'EAbnormalEffectType::CastSkill') {
        continue;
      }

      const child = Number(row.Values[1]);

      for (const parent of this.#sources.get(row.AbnormalId.Value) ?? []) {
        this.link(parent, child);
      }
    }
  }

  propagate() {
    const queue = [...new Set([...this.#classes.keys(), ...this.#public])];

    for (let position = 0; position < queue.length; position++) {
      const parent = queue[position];

      for (const child of this.#children.get(parent) ?? []) {
        const changes = [...this.classes(parent)].map((name) => this.addClass(child, name));
        const changed = changes.some((value) => value === true);
        const publicChild = this.#public.has(parent) && this.#public.has(child) === false;

        if (publicChild === true) {
          this.#public.add(child);
        }

        if (changed === true || publicChild === true) {
          queue.push(child);
        }
      }
    }
  }

  alternatives() {
    const rows = this.#tables.table('Skill');
    const signatures = new Set(
      rows
        .filter((row) => this.#public.has(row.ID.Value))
        .map((row) => `${row.SkillString_Key}|${row.GroupCoolTimeId}`),
    );

    for (const row of rows) {
      const signature = `${row.SkillString_Key}|${row.GroupCoolTimeId}`;

      if (row.GroupCoolTimeId !== 0 && signatures.has(signature) === true) {
        this.#public.add(row.ID.Value);
      }
    }
  }

  families() {
    const rows = new Map(this.#tables.table('Skill').map((row) => [row.ID.Value, row]));
    const groups = new Map();

    for (const id of this.#public) {
      const root = this.specializationRoot(id);
      const row = rows.get(root);
      const metadata = this.#runtime.skills[root];

      if (
        row === undefined ||
        metadata.cooldownGroup === null ||
        this.#text.named('skill', row) === false
      ) {
        continue;
      }

      this.append(groups, this.familySignature(row), root);
    }

    for (const roots of groups.values()) {
      const ids = [...roots].sort((left, right) => left - right);
      const primary = ids.find((id) => this.#runtime.skills[id].cooldownGroup === id) ?? ids[0];

      ids.forEach((id) => this.#aliases.set(id, primary));

      const classes = new Set(ids.flatMap((id) => [...this.classes(id)]));

      ids.forEach((id) => classes.forEach((name) => this.addClass(id, name)));
    }
  }

  familySignature(row) {
    return JSON.stringify([
      row.GroupCoolTimeId,
      row.SkillIcon,
      row.SkillType,
      this.#text.name('skill', row, 'en'),
      this.#text.name('skill', row, 'ru'),
    ]);
  }

  addClass(id, name) {
    if (this.#supported.has(name) === false || this.#runtime.skills[id] === undefined) {
      return false;
    }

    const values = this.#classes.get(id) ?? new Set();
    const changed = values.has(name) === false;

    values.add(name);
    this.#classes.set(id, values);

    return changed;
  }

  link(parent, child) {
    if (
      parent === child ||
      this.#runtime.skills[parent] === undefined ||
      this.#runtime.skills[child] === undefined
    ) {
      return;
    }

    this.append(this.#children, parent, child);
  }

  append(map, key, value) {
    const values = map.get(key) ?? new Set();

    values.add(value);
    map.set(key, values);
  }

  root(id) {
    const primary = this.specializationRoot(id);

    return this.#aliases.get(primary) ?? primary;
  }

  specializationRoot(id, visited = new Set()) {
    const parents = this.#parents.get(id);

    if (parents === undefined || parents.size !== 1) {
      return id;
    }

    if (visited.has(id) === true) {
      throw new Error('Cyclic specialization family: ' + id);
    }

    visited.add(id);

    return this.specializationRoot([...parents][0], visited);
  }

  classes(id) {
    return this.#classes.get(id) ?? new Set();
  }

  player(id) {
    return this.#common.has(id) || this.#public.has(id);
  }

  sources(id) {
    return [...(this.#sources.get(id) ?? [])].sort((a, b) => a - b);
  }

  effectClasses(id) {
    return new Set(
      this.sources(id)
        .filter((skill) => this.player(skill))
        .flatMap((skill) => [...this.classes(skill)]),
    );
  }

  effectFamily(row, text) {
    const sources = this.sources(row.ID.Value);
    const playerSources = sources.filter((id) => this.player(id));
    const applicable = playerSources.length > 0 ? playerSources : sources;
    const roots = [...new Set(applicable.map((id) => this.root(id)))].sort((a, b) => a - b);

    if (roots.length === 0) {
      return 'id:' + row.ID.Value;
    }

    return JSON.stringify([
      row.GroupId,
      row.AbnormalIcon,
      row.AbnormalType,
      text.name('effect', row, 'en'),
      text.name('effect', row, 'ru'),
      roots,
    ]);
  }
}
