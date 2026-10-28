import { shallowReadonly, shallowRef } from 'vue';
import type { ShallowRef } from 'vue';
import type { CatalogClient } from './catalog-client';
import type { CatalogSearch } from '../model/catalog-search';
import type { CatalogScope } from '../model/catalog-scope';
import type { CatalogItem } from '../model/catalog-item';
import type { CatalogPickerState } from './catalog-picker-state';

export class CatalogPickerController {
  private readonly state = shallowRef<CatalogPickerState>({
    page: { catalogVersion: '', total: 0, availableCategories: [], items: [] },
    selected: null,
    loading: false,
    restoring: false,
    error: null,
  });

  private sequence = 0;
  private restoreSequence = 0;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private request: CatalogSearch | null = null;

  public constructor(private readonly client: CatalogClient) {}

  public getState(): Readonly<ShallowRef<CatalogPickerState>> {
    return shallowReadonly(this.state);
  }

  public schedule(request: CatalogSearch): void {
    const sequence = ++this.sequence;

    this.request = { ...request, offset: 0 };

    if (this.timer !== null) {
      clearTimeout(this.timer);
    }

    this.state.value = { ...this.state.value, loading: true };
    this.timer = setTimeout(() => {
      void this.search({ ...request, offset: 0 }, sequence, false);
    }, 150);
  }

  public async more(): Promise<void> {
    const state = this.state.value;

    if (
      this.request === null ||
      state.loading === true ||
      state.page.items.length >= state.page.total === true
    ) {
      return;
    }

    this.state.value = { ...state, loading: true };

    await this.search({ ...this.request, offset: state.page.items.length }, ++this.sequence, true);
  }

  private async search(request: CatalogSearch, sequence: number, append: boolean): Promise<void> {
    try {
      const page = await this.client.search(request);

      if (sequence !== this.sequence) {
        return;
      }

      const previous = this.state.value.page;

      const items = append ? [...previous.items, ...page.items] : page.items;

      this.state.value = {
        ...this.state.value,
        page: { ...page, items },
        loading: false,
        error: null,
      };
    } catch (error) {
      if (sequence === this.sequence) {
        this.state.value = { ...this.state.value, loading: false, error: String(error) };
      }
    }
  }

  public select(item: CatalogItem): void {
    this.restoreSequence++;
    this.state.value = { ...this.state.value, selected: item, restoring: false };
  }

  public async restore(
    scope: CatalogScope,
    contextKey: string | null,
    key: string | null,
  ): Promise<void> {
    const sequence = ++this.restoreSequence;

    this.state.value = { ...this.state.value, selected: null, restoring: key !== null };

    if (key === null) {
      return;
    }

    try {
      const record = await this.client.item(scope, contextKey, key);

      if (sequence === this.restoreSequence) {
        this.state.value = {
          ...this.state.value,
          selected: record.item,
          restoring: false,
          page: { ...this.state.value.page, catalogVersion: record.catalogVersion },
        };
      }
    } catch (error) {
      if (sequence === this.restoreSequence) {
        this.state.value = { ...this.state.value, restoring: false, error: String(error) };
      }
    }
  }

  public dispose(): void {
    this.sequence++;
    this.restoreSequence++;

    if (this.timer !== null) {
      clearTimeout(this.timer);
    }
  }
}
