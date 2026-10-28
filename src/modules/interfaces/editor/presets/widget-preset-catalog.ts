import { CatalogScope } from '../../model/catalog-scope';
import { EffectFilter } from '../../model/rules/effect-filter';
import { WidgetKind } from '../../model/widget-kind';
import { WidgetPreset } from './widget-preset';

export class WidgetPresetCatalog {
  private readonly entries: Readonly<Record<WidgetKind, readonly WidgetPreset[]>> = {
    [WidgetKind.Icon]: [
      WidgetPreset.Empty,
      WidgetPreset.Cooldown,
      WidgetPreset.Ready,
      WidgetPreset.Buff,
      WidgetPreset.Debuff,
      WidgetPreset.EnemyCast,
      WidgetPreset.EventCountdown,
    ],
    [WidgetKind.Bar]: [
      WidgetPreset.Empty,
      WidgetPreset.Health,
      WidgetPreset.Mana,
      WidgetPreset.Stamina,
      WidgetPreset.Cast,
      WidgetPreset.EnemyCast,
      WidgetPreset.EventCountdown,
    ],
    [WidgetKind.Text]: [
      WidgetPreset.Empty,
      WidgetPreset.HealthPercent,
      WidgetPreset.Name,
      WidgetPreset.CastTimer,
    ],
    [WidgetKind.Group]: [WidgetPreset.Empty],
    [WidgetKind.DynamicGroup]: [WidgetPreset.Empty, WidgetPreset.Buffs],
  };

  private readonly units = new Set([
    WidgetPreset.Health,
    WidgetPreset.Mana,
    WidgetPreset.Cast,
    WidgetPreset.HealthPercent,
    WidgetPreset.Name,
    WidgetPreset.CastTimer,
  ]);

  private readonly scopes: Partial<Readonly<Record<WidgetPreset, CatalogScope>>> = {
    [WidgetPreset.Cooldown]: CatalogScope.Skills,
    [WidgetPreset.Ready]: CatalogScope.Skills,
    [WidgetPreset.Buff]: CatalogScope.Effects,
    [WidgetPreset.Debuff]: CatalogScope.Effects,
  };

  private readonly filters: Partial<Readonly<Record<WidgetPreset, EffectFilter>>> = {
    [WidgetPreset.Buff]: EffectFilter.Buff,
    [WidgetPreset.Debuff]: EffectFilter.Debuff,
  };

  public list(kind: WidgetKind): readonly WidgetPreset[] {
    return this.entries[kind];
  }

  public scope(preset: WidgetPreset): CatalogScope | null {
    return this.scopes[preset] ?? null;
  }

  public selectableUnit(preset: WidgetPreset): boolean {
    return this.units.has(preset);
  }

  public effectFilter(preset: WidgetPreset): EffectFilter | null {
    return this.filters[preset] ?? null;
  }

  public isEnemy(preset: WidgetPreset): boolean {
    return preset === WidgetPreset.EnemyCast || preset === WidgetPreset.EventCountdown;
  }
}
