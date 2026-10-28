import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import path from 'node:path';
import prettier from 'prettier';
import { ClientTables } from './catalog/client-tables.mjs';
import { CatalogText } from './catalog/catalog-text.mjs';
import { CatalogRelations } from './catalog/catalog-relations.mjs';
import { CatalogRecords } from './catalog/catalog-records.mjs';
import { PlayerCatalog } from './catalog/player-catalog.mjs';
import { NpcBindings } from './catalog/npc-bindings.mjs';
import { EnemyCatalog } from './catalog/enemy-catalog.mjs';

export class CatalogRebuild {
  #repository = path.resolve(import.meta.dirname, '..');
  #files = new Map();

  read(filename) {
    return JSON.parse(readFileSync(filename, 'utf8'));
  }

  base() {
    const configuration = this.read(path.join(this.#repository, 'data/application.json'));
    const directory = path.join(this.#repository, 'data', configuration.catalog.directory);
    const manifest = this.read(path.join(directory, configuration.catalog.manifest));

    for (const name of Object.keys(manifest.files)) {
      this.#files.set(name, this.read(path.join(directory, name)));
    }

    return manifest;
  }

  choices() {
    const result = new Map();
    const player = this.#files.get('player.json');
    const enemies = this.#files.get('enemies.json');

    this.collect(result, 'skills', '', player.skills);
    this.collect(result, 'effects', '', player.effects);

    for (const row of enemies.entries) {
      this.collect(result, 'enemies', '', [row.entry]);
      this.collect(result, 'enemy_skills', row.entry.key, row.skills);
      this.collect(result, 'enemy_effects', row.entry.key, row.effects);
    }

    return result;
  }

  collect(result, scope, context, entries) {
    for (const entry of entries) {
      this.collectForms(result, [scope, context, entry.key].join('|'), entry.forms);
    }
  }

  collectForms(result, prefix, forms) {
    for (const form of forms) {
      const key = prefix + '|' + form.key;

      if (result.has(key) === true) {
        throw new Error('Duplicate catalog choice: ' + key);
      }

      result.set(key, [...form.matchIds]);
    }
  }

  preserve(before) {
    const after = this.choices();

    for (const [key, ids] of before) {
      if (ids.every((id) => after.get(key)?.includes(id) === true) === false) {
        throw new Error('Existing profile choice would lose IDs: ' + key);
      }
    }

    return { choicesBefore: before.size, choicesAfter: after.size, removedChoices: 0 };
  }

  labels(tables, text) {
    for (const row of tables.table('Skill')) {
      if (text.named('skill', row) === true) {
        text.labelEntry(`skill-id:${row.ID.Value}`, 'skill', row);
        text.labelEntry(`npc-skill:${row.ID.Value}`, 'skill', row);
      }
    }

    for (const [table, kind, prefix] of [
      ['SkillAbnormal', 'effect', 'effect-id'],
      ['NpcData', 'npc', 'npc'],
    ]) {
      this.labelTable(tables.table(table), kind, prefix, text);
    }
  }

  labelTable(rows, kind, prefix, text) {
    for (const row of rows) {
      if (text.named(kind, row) === true) {
        text.labelEntry(`${prefix}:${row.ID.Value}`, kind, row);
      }
    }
  }

  summary() {
    const player = this.#files.get('player.json');
    const enemies = this.#files.get('enemies.json').entries;
    const ids = (entries) =>
      new Set(entries.flatMap((entry) => entry.forms.flatMap((form) => form.matchIds))).size;

    return {
      playerSkills: player.skills.length,
      playerSkillIds: ids(player.skills),
      playerEffects: player.effects.length,
      playerEffectIds: ids(player.effects),
      enemies: enemies.length,
      npcIds: ids(enemies.map((row) => row.entry)),
      enemySkillChoices: enemies.reduce((count, row) => count + row.skills.length, 0),
    };
  }

  async write(output, manifest, version, report) {
    mkdirSync(output, { recursive: true });
    const files = {};

    for (const [name, value] of this.#files) {
      value.catalogVersion = version;
      const text = await prettier.format(JSON.stringify(value, null, 2), {
        parser: 'json',
        printWidth: 100,
      });
      const bytes = Buffer.from(text);

      mkdirSync(path.dirname(path.join(output, name)), { recursive: true });
      writeFileSync(path.join(output, name), bytes, { flag: 'wx' });
      files[name] = {
        sha256: createHash('sha256').update(bytes).digest('hex'),
        bytes: bytes.length,
      };
    }

    const updated = { ...manifest, catalogVersion: version, files };

    writeFileSync(path.join(output, 'manifest.json'), JSON.stringify(updated, null, 2) + '\n', {
      flag: 'wx',
    });
    writeFileSync(
      path.join(output, 'rebuild-report.json'),
      JSON.stringify(report, null, 2) + '\n',
      { flag: 'wx' },
    );
  }

  async run(source, output, version) {
    if (source === undefined || output === undefined) {
      throw new Error(
        'Usage: node checks/catalog-rebuild.mjs <extracted client> <new output directory> [catalog version]',
      );
    }

    if (existsSync(output) === true) {
      throw new Error('Output directory already exists: ' + output);
    }

    const manifest = this.base();
    const before = this.choices();
    const previous = this.summary();
    const tables = new ClientTables(path.resolve(source));
    const runtime = this.#files.get('runtime.json');
    const observed = this.read(path.join(import.meta.dirname, 'catalog/observed-skills.json'));

    if (observed.clientBuild !== runtime.clientBuild) {
      throw new Error('Observed NPC abilities belong to a different client build');
    }

    if (observed.playerSkills.some((id) => runtime.skills[id] === undefined) === true) {
      throw new Error('Observed player ability is not a client definition');
    }

    tables.validate(runtime);
    const text = new CatalogText({
      en: this.#files.get('translations/en.json'),
      ru: this.#files.get('translations/ru.json'),
    });
    const playerIds = this.#files
      .get('player.json')
      .skills.flatMap((entry) => entry.forms.flatMap((form) => form.matchIds));
    const relations = new CatalogRelations(
      tables,
      runtime,
      [...playerIds, ...observed.playerSkills],
      text,
    );
    const records = new CatalogRecords(text, runtime);
    const bindings = new NpcBindings(tables, relations, observed);

    text.load(tables);
    relations.load();
    bindings.load(runtime);
    new PlayerCatalog(tables, text, relations, records, runtime).extend(
      this.#files.get('player.json'),
    );
    new EnemyCatalog(tables, text, records, bindings, runtime).extend(
      this.#files.get('enemies.json'),
    );
    this.labels(tables, text);

    const report = {
      clientBuild: runtime.clientBuild,
      catalogVersion: version ?? manifest.catalogVersion,
      previous,
      current: this.summary(),
      compatibility: this.preserve(before),
      sourceHashes: tables.hashes(),
      observedNpcSkills: observed.npcSkills,
      observedPlayerSkills: observed.playerSkills,
    };

    await this.write(path.resolve(output), manifest, report.catalogVersion, report);
    console.log(JSON.stringify({ ...report, sourceHashes: undefined }, null, 2));
  }
}

await new CatalogRebuild().run(...process.argv.slice(2));
