import type { CatalogField } from './catalog-field';
import { DataKind } from './data-kind';

export const bossCatalog: readonly CatalogField[] = [
  { path: 'bossId', kind: DataKind.Number },
  { path: 'mapId', kind: DataKind.Number },
  { path: 'name', kind: DataKind.Text },
  { path: 'known', kind: DataKind.Boolean },
  { path: 'alive', kind: DataKind.Boolean },
  { path: 'observedAlive', kind: DataKind.Boolean },
  { path: 'restored', kind: DataKind.Boolean },
  { path: 'expected', kind: DataKind.Boolean },
  { path: 'respawnsAt', kind: DataKind.Timestamp },
  { path: 'countdown', kind: DataKind.Duration },
  { path: 'duration', kind: DataKind.Duration },
  { path: 'elapsed', kind: DataKind.Duration },
  { path: 'remaining', kind: DataKind.Duration },
];
