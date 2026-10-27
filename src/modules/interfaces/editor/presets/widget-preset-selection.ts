import type { CatalogItem } from '../../model/catalog-item';
import type { CatalogSelection } from '../../model/catalog-selection';

export interface WidgetPresetSelection {
  readonly item: CatalogItem;
  readonly selection: CatalogSelection;
}
