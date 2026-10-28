import type { CatalogItem } from '../model/catalog-item';
import type { UiSelectOption } from '../../../renderer/components/ui/ui-select-option';
import { CatalogScope } from '../model/catalog-scope';

export class CatalogOptions {
  public constructor(
    private readonly translate: (key: string) => string,
    private readonly locale: () => string,
  ) {}

  public list(
    items: readonly CatalogItem[],
    scope: CatalogScope,
    grouped: boolean,
  ): readonly UiSelectOption<string>[] {
    const options = items.map((item) => ({
      value: item.key,
      label: item.context === '' ? item.name : item.name + ' · ' + item.context,
      ...(grouped ? { group: this.group(item, scope) } : {}),
    }));

    if (grouped === false) {
      return options;
    }

    return options.sort((left, right) =>
      (left.group ?? '').localeCompare(right.group ?? '', this.locale()),
    );
  }

  private group(item: CatalogItem, scope: CatalogScope): string {
    if (item.sourceClasses.length > 1 === true) {
      return this.translate(
        scope === CatalogScope.Skills
          ? 'modules.interfaces.catalogGroups.sharedSkills'
          : 'modules.interfaces.catalogGroups.sharedEffects',
      );
    }

    if (item.sourceClasses.length === 1) {
      return item.sourceClasses
        .map((value) => this.translate('modules.interfaces.classes.' + value))
        .join(' / ');
    }

    return this.translate(
      scope === CatalogScope.Skills
        ? 'modules.interfaces.catalogGroups.commonSkills'
        : 'modules.interfaces.catalogGroups.otherEffects',
    );
  }
}
