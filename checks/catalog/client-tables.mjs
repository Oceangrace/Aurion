import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import path from 'node:path';

export class ClientTables {
  #root;
  #tables = new Map();
  #hashes = {};

  constructor(root) {
    this.#root = root;
  }

  read(relative) {
    const bytes = readFileSync(path.join(this.#root, relative));

    this.#hashes[relative] = createHash('sha256').update(bytes).digest('hex');

    return JSON.parse(bytes.toString('utf8').replace(/^\uFEFF/, ''));
  }

  table(name) {
    if (this.#tables.has(name) === false) {
      const relative = `decoded/AION2/Content/Data/Table/${name}.json`;
      const rows = this.read(relative).Properties?.Data;

      if (Array.isArray(rows) === false) {
        throw new Error('Decoded table is missing: ' + name);
      }

      this.#tables.set(name, rows);
    }

    return this.#tables.get(name);
  }

  locale(language) {
    return this.read(`decrypted/AION2/Content/L10N/Text/${language}/L10NString.json`).Entries;
  }

  validate(runtime) {
    const source = this.read('../client-source.json');
    const build = /<Version>(\d+)<\/Version>/.exec(source.versionXml ?? '');

    if (Number(build?.[1]) !== runtime.clientBuild) {
      throw new Error('Client snapshot build does not match the base catalog');
    }

    for (const [table, field] of [
      ['Skill', 'skills'],
      ['SkillAbnormal', 'effects'],
      ['NpcData', 'npcs'],
    ]) {
      const ids = new Set(this.table(table).map((row) => row.ID.Value));
      const expected = Object.keys(runtime[field]).map(Number);

      if (ids.size !== expected.length || expected.some((id) => ids.has(id) === false) === true) {
        throw new Error('Client definitions do not match the base catalog: ' + table);
      }
    }
  }

  hashes() {
    return { ...this.#hashes };
  }
}
