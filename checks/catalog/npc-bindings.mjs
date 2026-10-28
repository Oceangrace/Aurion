export class NpcBindings {
  #tables;
  #relations;
  #observed;
  #skills = new Map();
  #effects = new Map();

  constructor(tables, relations, observed) {
    this.#tables = tables;
    this.#relations = relations;
    this.#observed = observed;
  }

  load(runtime) {
    const behaviors = new Map();

    for (const row of this.#tables.table('NpcBehaviorCase')) {
      const rows = behaviors.get(row.NpcName) ?? [];

      rows.push(row);
      behaviors.set(row.NpcName, rows);
    }

    for (const row of this.#tables.table('NpcData')) {
      this.npc(row, behaviors.get(row.Name) ?? [], runtime);
    }

    this.linkEffects();
  }

  npc(row, behaviors, runtime) {
    const id = row.ID.Value;
    const skills = new Set(row.ActiveSkillList.map((skill) => skill.Value));
    const effects = new Set();

    for (const behavior of behaviors) {
      const value = Number(behavior.Value);

      if (
        ['ENpcBehaviorCase::Casting', 'ENpcBehaviorCase::SkillUse'].includes(behavior.Case) === true
      ) {
        skills.add(value);
      }

      if (behavior.Case === 'ENpcBehaviorCase::Abnormal') {
        effects.add(value);
      }
    }

    for (const skill of this.#observed.npcSkills[id] ?? []) {
      if (runtime.skills[skill] === undefined) {
        throw new Error('Observed ability is not a client definition: ' + skill);
      }

      skills.add(skill);
    }

    this.#skills.set(
      id,
      new Set([...skills].filter((skill) => runtime.skills[skill] !== undefined)),
    );
    this.#effects.set(
      id,
      new Set([...effects].filter((effect) => runtime.effects[effect] !== undefined)),
    );
  }

  linkEffects() {
    const bySkill = new Map();

    for (const row of this.#tables.table('SkillAbnormal')) {
      for (const skill of this.#relations.sources(row.ID.Value)) {
        const effects = bySkill.get(skill) ?? new Set();

        effects.add(row.ID.Value);
        bySkill.set(skill, effects);
      }
    }

    for (const [id, skills] of this.#skills) {
      const effects = this.#effects.get(id);

      skills.forEach((skill) =>
        (bySkill.get(skill) ?? []).forEach((effect) => effects.add(effect)),
      );
    }
  }

  skills(id) {
    return this.#skills.get(id) ?? new Set();
  }

  effects(id) {
    return this.#effects.get(id) ?? new Set();
  }

  observed(id) {
    return this.#observed.npcSkills[id] !== undefined;
  }

  shared(ids, kind) {
    const sources = kind === 'skill' ? this.#skills : this.#effects;
    const first = sources.get(ids[0]) ?? new Set();

    return [...first].filter((id) => ids.every((npc) => sources.get(npc)?.has(id) === true));
  }
}
