import type { WidgetNode } from '../../model/widget-node';
import type { WidgetPresetOptions } from './widget-preset-options';
import { WidgetPreset } from './widget-preset';
import { ObservationKind } from '../../model/observation-kind';
import { TriggerSubject } from '../../model/trigger-subject';
import { TriggerType } from '../../model/rules/trigger-type';
import { TriggerUnit } from '../../model/rules/trigger-unit';
import { EffectFilter } from '../../model/rules/effect-filter';
import { EffectPresence } from '../../model/rules/effect-presence';
import { EffectOrigin } from '../../model/rules/effect-origin';
import { CloneDirection } from '../../model/rules/clone-direction';
import { LogicMode } from '../../model/rules/logic-mode';

export class IconWidgetPresets {
  public apply(node: WidgetNode, preset: WidgetPreset, options: WidgetPresetOptions): WidgetNode {
    const chosen = options.catalog;

    if (chosen === null) {
      return node;
    }

    const widget = {
      ...node,
      name: chosen.item.name,
      appearance: { ...node.appearance, iconPath: chosen.item.icon },
      style: { ...node.style, showTimer: options.showTimer },
    };

    if (preset === WidgetPreset.Buff || preset === WidgetPreset.Debuff) {
      return this.effect(widget, preset, options);
    }

    return {
      ...widget,
      trigger: {
        subject: TriggerSubject.SelfPlayer,
        observation:
          preset === WidgetPreset.Ready ? ObservationKind.SkillReady : ObservationKind.Cooldown,
        actorKey: null,
        selection: chosen.selection,
        displaySeconds: null,
      },
    };
  }

  private effect(node: WidgetNode, preset: WidgetPreset, options: WidgetPresetOptions): WidgetNode {
    const chosen = options.catalog;

    if (chosen === null) {
      return node;
    }

    const id = crypto.randomUUID();

    const debuff = preset === WidgetPreset.Debuff;

    return {
      ...node,
      ruleSelections: { [id]: [chosen.selection] },
      rules: {
        mode: LogicMode.All,
        dataRuleId: id,
        rules: [
          {
            id,
            kind: TriggerType.Effect,
            observe: false,
            trigger: {
              unit: debuff ? TriggerUnit.Target : TriggerUnit.Player,
              member: '',
              filter: debuff ? EffectFilter.Debuff : EffectFilter.Buff,
              effects: [],
              presence: EffectPresence.Present,
              origin: EffectOrigin.Any,
              minStacks: 1,
              remainingBelowSeconds: 0,
              clones: false,
              cloneDirection: CloneDirection.Right,
            },
          },
        ],
      },
    };
  }
}
