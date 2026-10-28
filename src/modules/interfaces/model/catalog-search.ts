import type { CatalogScope } from './catalog-scope';
import type { CatalogSource } from './catalog-source';

export interface CatalogSearch {
  readonly offset?: number;
  readonly source?: CatalogSource;
  readonly matchIds?: readonly number[];
  readonly scope: CatalogScope;
  readonly contextKey: string | null;
  readonly text: string;
  readonly category: string | null;
}
