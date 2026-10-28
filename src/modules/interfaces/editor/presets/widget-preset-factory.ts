import type { WidgetNode } from '../../model/widget-node';
import type { WidgetPresetOptions } from './widget-preset-options';
import { WidgetFactory } from '../widget-factory';
import { WidgetPreset } from './widget-preset';
import { WidgetKind } from '../../model/widget-kind';
import { WidgetPresetCatalog } from './widget-preset-catalog';
import { IconWidgetPresets } from './icon-widget-presets';
import { ResourceWidgetPresets } from './resource-widget-presets';
import { EnemyWidgetPresets } from './enemy-widget-presets';
import { defaultGroupDataBinding } from '../../model/rules/group-data-binding';
import { defaultGroupEffectSettings } from '../../model/rules/group-effect-settings';
import { GroupDataSource } from '../../model/rules/group-data-source';

export class WidgetPresetFactory {
  private readonly factory = new WidgetFactory();
  private readonly catalog = new WidgetPresetCatalog();
  private readonly icons = new IconWidgetPresets();
  private readonly resources = new ResourceWidgetPresets();
  private readonly enemies = new EnemyWidgetPresets();

  public create(
    kind: WidgetKind,
    preset: WidgetPreset,
    name: string,
    options: WidgetPresetOptions,
  ): WidgetNode | null {
    if (this.catalog.list(kind).includes(preset) === false) {
      return null;
    }

    const node = this.factory.create(kind, name);

    if (this.catalog.isEnemy(preset) === true) {
      return this.enemies.apply(node, preset, options);
    }

    if (this.catalog.scope(preset) !== null && options.catalog === null) {
      return null;
    }

    if (preset === WidgetPreset.Empty) {
      return node;
    }

    return this.apply(node, preset, options);
  }

  private apply(node: WidgetNode, preset: WidgetPreset, options: WidgetPresetOptions): WidgetNode {
    switch (node.kind) {
      case WidgetKind.Icon:
        return this.icons.apply(node, preset, options);
      case WidgetKind.Text:
      case WidgetKind.Bar:
        return this.resources.apply(node, preset, options);
      case WidgetKind.DynamicGroup:
        return {
          ...node,
          layout: { ...node.layout, width: 240, height: 40 },
          binding: {
            ...defaultGroupDataBinding,
            source: GroupDataSource.Effects,
            effects: { ...defaultGroupEffectSettings },
          },
        };
      case WidgetKind.Group:
        return node;
    }
  }
}
