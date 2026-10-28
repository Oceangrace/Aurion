import type { UiSelectConfiguration } from './ui-select-configuration';
import type { UiSelectMenuState } from './ui-select-menu-state';
import type { UiSelectOption } from './ui-select-option';
import type { UiSelectSection } from './ui-select-section';

export class UiSelectMenu<T extends string> {
  public constructor(
    private readonly configuration: UiSelectConfiguration<T>,
    private readonly state: UiSelectMenuState,
  ) {}

  public reset(): void {
    const selected = this.configuration.options.find(
      (option) => option.value === this.configuration.modelValue,
    );

    this.state.query = '';
    this.state.limit = 80;
    const parent = selected?.parentGroup ?? selected?.group ?? '';

    this.state.expanded = [parent, this.childKey(parent, selected?.group ?? '')];
  }

  public search(query: string): void {
    this.state.query = query;
    this.state.limit = 80;
  }

  public toggle(group: string): void {
    this.state.expanded = this.state.expanded.includes(group)
      ? this.state.expanded.filter((value) => value !== group)
      : [...this.state.expanded, group];
  }

  public more(): void {
    this.state.limit += 80;
  }

  public sections(): readonly UiSelectSection<T>[] {
    const groups = new Map<string, UiSelectOption<T>[]>();

    const query =
      this.configuration.remoteSearch === true ? '' : this.normalize(this.state.query.trim());

    for (const option of this.configuration.options) {
      const group = option.parentGroup ?? option.group ?? '';

      if (
        this.normalize(
          option.label + ' ' + option.value + ' ' + group + ' ' + (option.group ?? ''),
        ).includes(query) === true
      ) {
        const entries = groups.get(group) ?? [];

        entries.push(option);
        groups.set(group, entries);
      }
    }

    return [...groups].map(([label, options]) => this.parent(label, options));
  }

  public visible(sections: readonly UiSelectSection<T>[]): readonly UiSelectOption<T>[] {
    return sections.flatMap((section) => [...section.options, ...this.visible(section.children)]);
  }

  private parent(label: string, options: readonly UiSelectOption<T>[]): UiSelectSection<T> {
    const expanded = this.expanded(label);

    const nested = options.some(
      (option) => option.parentGroup !== undefined && option.group !== label,
    );

    if (nested === false) {
      return this.leaf(label, label, options);
    }

    const groups = new Map<string, UiSelectOption<T>[]>();

    for (const option of options) {
      const group = option.group ?? '';

      const entries = groups.get(group) ?? [];

      entries.push(option);
      groups.set(group, entries);
    }

    const children = expanded
      ? [...groups].map(([child, entries]) =>
          this.leaf(this.childKey(label, child), child, entries),
        )
      : [];

    return { key: label, label, expanded, total: options.length, options: [], children };
  }

  private leaf(
    key: string,
    label: string,
    options: readonly UiSelectOption<T>[],
  ): UiSelectSection<T> {
    const expanded = label === '' || this.expanded(key);

    return {
      key,
      label,
      expanded,
      total: options.length,
      options: expanded ? this.options(options) : [],
      children: [],
    };
  }

  private options(options: readonly UiSelectOption<T>[]): readonly UiSelectOption<T>[] {
    return this.configuration.remoteSearch === true ? options : options.slice(0, this.state.limit);
  }

  private expanded(key: string): boolean {
    return this.state.query.trim() !== '' || this.state.expanded.includes(key);
  }

  private childKey(parent: string, child: string): string {
    return parent + '\u001f' + child;
  }

  private normalize(value: string): string {
    return value.toLowerCase().replaceAll('ё', 'е');
  }
}
