import { readFileSync } from 'node:fs';
import path from 'node:path';

export class CatalogKeys {
  constructor(previous, current) {
    this.previous = previous;
    this.current = current;
  }

  read(directory, name) {
    return JSON.parse(readFileSync(path.join(directory, name), 'utf8'));
  }

  add(result, scope, context, item) {
    for (const form of item.forms) {
      result.add([scope, context, item.key, form.key].join(' | '));
    }
  }

  keys(directory) {
    const result = new Set();
    const player = this.read(directory, 'player.json');
    const enemies = this.read(directory, 'enemies.json');

    for (const item of player.skills) {
      this.add(result, 'skills', '', item);
    }

    for (const item of player.effects) {
      this.add(result, 'effects', '', item);
    }

    for (const enemy of enemies.entries) {
      this.add(result, 'enemies', '', enemy.entry);

      for (const item of enemy.skills) {
        this.add(result, 'enemy-skills', enemy.entry.key, item);
      }

      for (const item of enemy.effects) {
        this.add(result, 'enemy-effects', enemy.entry.key, item);
      }
    }

    return result;
  }

  run() {
    if (this.previous === undefined || this.current === undefined) {
      console.error(
        'Usage: node checks/catalog-keys.mjs <old catalog directory> <new catalog directory>',
      );

      return 2;
    }

    const before = this.keys(this.previous);
    const after = this.keys(this.current);
    const removed = [...before].filter((key) => after.has(key) === false).sort();

    console.log(
      `Catalog keys: ${String(before.size)} before, ${String(after.size)} after, ${String(removed.length)} removed.`,
    );

    for (const key of removed) {
      console.log('  removed: ' + key);
    }

    return removed.length === 0 ? 0 : 1;
  }
}

process.exitCode = new CatalogKeys(...process.argv.slice(2)).run();
