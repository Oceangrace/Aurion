import type { DataKind } from './data-kind';
import type { FieldCategory } from './field-category';

export interface CatalogField {
  readonly path: string;
  readonly kind: DataKind;
  readonly category?: FieldCategory;
}
