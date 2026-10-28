import type { WidgetNode } from '../../model/widget-node';
import type { ResourceWidgetPreset } from './resource-widget-preset';
import type { WidgetPresetOptions } from './widget-preset-options';
import { WidgetPreset } from './widget-preset';
import { ProgressSource } from '../../model/progress-source';
import { TriggerUnit } from '../../model/rules/trigger-unit';
import { TriggerType } from '../../model/rules/trigger-type';
import { ComparisonOperator } from '../../model/rules/comparison-operator';
import { LogicMode } from '../../model/rules/logic-mode';
import { WidgetKind } from '../../model/widget-kind';

export class ResourceWidgetPresets {
  private readonly entries: Partial<Readonly<Record<WidgetPreset, ResourceWidgetPreset>>> = {
    [WidgetPreset.Health]: {
      path: 'hp.percent',
      text: '{unit.hp.value:0} / {unit.hp.max:0}',
      fill: '#ba6e70',
      background: '#302023',
      casting: false,
    },
    [WidgetPreset.Mana]: {
      path: 'mp.percent',
      text: '{unit.mp.value:0} / {unit.mp.max:0}',
      fill: '#588cae',
      background: '#1c2b35',
      casting: false,
    },
    [WidgetPreset.Stamina]: {
      path: 'stamina.percent',
      text: '{unit.stamina.value:0}',
      fill: '#c89b4b',
      background: '#292216',
      casting: false,
    },
    [WidgetPreset.Cast]: {
      path: 'cast.percent',
      text: '{unit.cast.remaining}',
      fill: '#42bbb9',
      background: '#1c3030',
      casting: true,
    },
    [WidgetPreset.HealthPercent]: {
      path: 'hp.percent',
      text: '{unit.hp.percent:0}%',
      fill: '#ba6e70',
      background: '#00000000',
      casting: false,
    },
    [WidgetPreset.Name]: {
      path: 'name',
      text: '{unit.name}',
      fill: '#42bbb9',
      background: '#00000000',
      casting: false,
    },
    [WidgetPreset.CastTimer]: {
      path: 'cast.remaining',
      text: '{unit.cast.remaining}',
      fill: '#42bbb9',
      background: '#00000000',
      casting: true,
    },
  };

  public apply(node: WidgetNode, preset: WidgetPreset, options: WidgetPresetOptions): WidgetNode {
    const resource = this.entries[preset];

    if (resource === undefined) {
      return node;
    }

    const unit = preset === WidgetPreset.Stamina ? TriggerUnit.Player : options.unit;

    return {
      ...node,
      rules: resource.casting ? this.castRule(unit) : null,
      appearance: {
        ...node.appearance,
        text: resource.text.replaceAll('unit.', unit + '.'),
        background: resource.background,
        fontSize: node.kind === WidgetKind.Bar ? 14 : 20,
      },
      style: {
        ...node.style,
        source: ProgressSource.Field,
        path: unit + '.' + resource.path,
        rangeMin: 0,
        rangeMax: 100,
        fillColor: resource.fill,
        borderWidth: 0,
        transitionMs: resource.casting ? 0 : 180,
      },
    };
  }

  private castRule(unit: TriggerUnit.Player | TriggerUnit.Target): WidgetNode['rules'] {
    return {
      mode: LogicMode.All,
      dataRuleId: null,
      rules: [
        {
          id: crypto.randomUUID(),
          kind: TriggerType.Value,
          observe: false,
          trigger: {
            path: unit + '.cast.active',
            operator: ComparisonOperator.Equal,
            value: 'true',
          },
        },
      ],
    };
  }
}
