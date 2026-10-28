import type { CatalogField } from '../model/rules/catalog-field';
import { FieldCategory } from '../model/rules/field-category';
import { FieldPath } from '../model/rules/field-path';
import { FieldScope } from '../model/rules/field-scope';
import type { FormulaDefinition } from '../model/rules/formula-definition';
import type { TriggerRule } from '../model/rules/trigger-rule';

export class FieldLabels {
  private readonly scopedGroups = new Set<FieldScope>([
    FieldScope.Formula,
    FieldScope.Party,
    FieldScope.Combat,
    FieldScope.Zone,
    FieldScope.Event,
    FieldScope.Trigger,
    FieldScope.Rule,
  ]);

  private readonly scopeTitles: Partial<Record<FieldScope, string>> = {
    [FieldScope.Rule]: 'modules.interfaces.rules.title',
    [FieldScope.Formula]: 'modules.interfaces.formulas.title',
    [FieldScope.Trigger]: 'modules.interfaces.data.trigger',
  };

  public constructor(
    private readonly translate: (key: string, params?: Record<string, unknown>) => string,
  ) {}

  public label(
    path: string,
    formulas: readonly FormulaDefinition[] = [],
    rules: readonly TriggerRule[] = [],
  ): string {
    const ruleLabel = this.ruleLabel(path, rules);

    if (ruleLabel !== null) {
      return ruleLabel;
    }

    const address = new FieldPath(path);

    if (address.scope() === FieldScope.Formula) {
      return (
        this.translate('modules.interfaces.formulas.title') +
        ' · ' +
        (formulas.find((formula) => formula.id === address.field())?.name ?? address.field())
      );
    }

    const field = address.field().replace('.', '_');

    const label = field.startsWith('stat_')
      ? this.translate('modules.interfaces.fieldNames.stat', { id: field.slice(5) })
      : this.translate('modules.interfaces.fieldNames.' + field);

    return this.scope(address.scope(), address.member()) + ' · ' + label;
  }

  public scope(scope: FieldScope | null, member: number | null = null): string {
    if (scope === null) {
      return '';
    }

    if (scope === FieldScope.Party) {
      return this.translate('modules.interfaces.data.party', { n: member });
    }

    return this.translate(this.scopeTitles[scope] ?? `modules.interfaces.data.scopes.${scope}`);
  }

  public group(field: CatalogField): string {
    const address = new FieldPath(field.path);

    const scope = address.scope();

    if (scope !== null && this.scopedGroups.has(scope) === true) {
      return this.scope(scope, address.member());
    }

    if (scope === FieldScope.Group) {
      return this.translate('modules.interfaces.fieldGroups.party');
    }

    return this.translate(
      'modules.interfaces.fieldGroups.' + (field.category ?? FieldCategory.State),
    );
  }

  public parentGroup(path: string): string {
    const address = new FieldPath(path);

    return address.scope() === FieldScope.Party
      ? this.scope(FieldScope.Group)
      : this.scope(address.scope());
  }

  private ruleLabel(path: string, rules: readonly TriggerRule[]): string | null {
    const index = rules.findIndex((rule) => path.startsWith('rule.' + rule.id + '.'));

    const rule = rules[index];

    if (rule === undefined) {
      return null;
    }

    const field = path.slice(('rule.' + rule.id + '.').length);

    if (field !== 'active') {
      return (
        String(index + 1) +
        ' · ' +
        this.translate('modules.interfaces.rules.types.' + rule.kind) +
        ' · ' +
        this.translate('modules.interfaces.fieldNames.' + field)
      );
    }

    return this.translate('modules.interfaces.conditions.ruleActive', {
      n: index + 1,
      kind: this.translate('modules.interfaces.rules.types.' + rule.kind),
    });
  }
}
