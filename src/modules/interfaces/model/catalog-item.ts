import type { CatalogForm } from './catalog-form';

export interface CatalogItem {
  readonly key: string;
  readonly name: string;
  readonly nameKnown?: boolean;
  readonly context: string;
  readonly icon: string | null;
  readonly categories: readonly string[];
  readonly sourceClasses: readonly string[];
  readonly forms: readonly CatalogForm[];
}
