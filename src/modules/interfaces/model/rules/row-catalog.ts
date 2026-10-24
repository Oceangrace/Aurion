import { combatStatisticsCatalog } from './combat-statistics-catalog';
import type { CatalogField } from './catalog-field';
import { DataKind } from './data-kind';
import { FieldCategory } from './field-category';

export const rowCatalog: readonly CatalogField[] = [
  { path: 'row.id', kind: DataKind.Text, category: FieldCategory.Identity },
  { path: 'row.name', kind: DataKind.Text, category: FieldCategory.Identity },
  { path: 'row.status', kind: DataKind.Text },
  { path: 'row.state', kind: DataKind.Text },
  { path: 'row.statusText', kind: DataKind.Text },
  { path: 'row.restored', kind: DataKind.Boolean },
  { path: 'row.timer', kind: DataKind.Text },
  { path: 'row.remaining', kind: DataKind.Duration },
  { path: 'row.observedAge', kind: DataKind.Duration },
  { path: 'row.mapId', kind: DataKind.Number },
  { path: 'row.slot', kind: DataKind.Number },
  { path: 'row.linked', kind: DataKind.Boolean },
  { path: 'row.rank', kind: DataKind.Number },
  { path: 'row.value', kind: DataKind.Number },
  ...combatStatisticsCatalog.map((field) => ({ ...field, path: 'row.' + field.path })),
  { path: 'row.entityId', kind: DataKind.Text },
  { path: 'row.skillId', kind: DataKind.Number },
  { path: 'row.class', kind: DataKind.Text, category: FieldCategory.Identity },
  { path: 'row.percent', kind: DataKind.Percent },
  { path: 'row.leaderPercent', kind: DataKind.Percent },
];
