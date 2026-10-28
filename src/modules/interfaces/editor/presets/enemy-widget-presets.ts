import type { WidgetNode } from '../../model/widget-node';
import type { TriggerRule } from '../../model/rules/trigger-rule';
import type { EventTriggerRule } from '../../model/rules/event-trigger-rule';
import type { WidgetPresetSelection } from './widget-preset-selection';
import type { WidgetPresetOptions } from './widget-preset-options';
import type { EnemyWidgetPresetOptions } from './enemy-widget-preset-options';
import { CatalogScope } from '../../model/catalog-scope';
import { GameEventType } from '../../model/rules/game-event-type';
import { EventParticipant } from '../../model/rules/event-participant';
import { TriggerType } from '../../model/rules/trigger-type';
import { LogicMode } from '../../model/rules/logic-mode';
import { ComparisonOperator } from '../../model/rules/comparison-operator';
import { ImageSource } from '../../model/image-source';
import { SoundCue } from '../../model/sound-cue';
import { WidgetPreset } from './widget-preset';
import { WidgetKind } from '../../model/widget-kind';
import { ProgressSource } from '../../model/progress-source';

export class EnemyWidgetPresets {
  public apply(
    node: WidgetNode,
    preset: WidgetPreset,
    options: WidgetPresetOptions,
  ): WidgetNode | null {
    const enemy = options.enemy;

    if (enemy === undefined || enemy.actorKey === null || enemy.skill === null) {
      return null;
    }

    if (this.selectionValid(enemy.skill, enemy.actorKey) === false) {
      return null;
    }

    if (preset === WidgetPreset.EventCountdown) {
      return this.countdown(node, options, enemy, enemy.skill);
    }

    const event = this.event(enemy, GameEventType.CastStarted, 3000);

    return this.widget(node, options, enemy, enemy.skill, event, []);
  }

  private countdown(
    node: WidgetNode,
    options: WidgetPresetOptions,
    enemy: EnemyWidgetPresetOptions,
    skill: WidgetPresetSelection,
  ): WidgetNode | null {
    const expected = enemy.expectedSkill;

    if (expected === null || this.selectionValid(expected, enemy.actorKey) === false) {
      return null;
    }

    if (this.timingValid(enemy) === false) {
      return null;
    }

    const duration = enemy.countdownSeconds * 1000;

    const event = this.event(enemy, GameEventType.SkillUsed, duration);

    const rule: EventTriggerRule = {
      ...event,
      trigger: { ...event.trigger, delayMs: duration },
    };

    return this.widget(
      node,
      options,
      enemy,
      expected,
      rule,
      [
        {
          id: crypto.randomUUID(),
          kind: TriggerType.Value,
          observe: false,
          trigger: {
            path: 'rule.' + rule.id + '.countdown',
            operator: ComparisonOperator.LessOrEqual,
            value: String(enemy.warningSeconds * 1000),
          },
        },
      ],
      skill,
    );
  }

  private timingValid(options: EnemyWidgetPresetOptions): boolean {
    return (
      [options.countdownSeconds, options.warningSeconds].every(
        (seconds) => Number.isFinite(seconds) && seconds >= 0.1 && seconds <= 3600,
      ) && options.warningSeconds <= options.countdownSeconds
    );
  }

  private selectionValid(selection: WidgetPresetSelection, actorKey: string | null): boolean {
    return (
      selection.selection.scope === CatalogScope.EnemySkills &&
      selection.selection.contextKey === actorKey
    );
  }

  private event(
    options: EnemyWidgetPresetOptions,
    event: GameEventType,
    durationMs: number,
  ): EventTriggerRule {
    return {
      id: crypto.randomUUID(),
      kind: TriggerType.Event,
      observe: false,
      trigger: {
        event,
        source: EventParticipant.Any,
        recipient: EventParticipant.Any,
        sourceActorKey: options.actorKey,
        skillId: null,
        effectId: null,
        minValue: null,
        durationMs,
        delayMs: 0,
        schedule: null,
      },
    };
  }

  private widget(
    node: WidgetNode,
    options: WidgetPresetOptions,
    enemy: EnemyWidgetPresetOptions,
    display: WidgetPresetSelection,
    event: EventTriggerRule,
    rules: readonly TriggerRule[],
    skill = display,
  ): WidgetNode {
    const widget: WidgetNode = {
      ...node,
      name: display.item.name,
      imageSelection: display.selection,
      ruleSelections: { [event.id]: [skill.selection] },
      rules: { mode: LogicMode.All, dataRuleId: event.id, rules: [event, ...rules] },
      appearance: { ...node.appearance, iconPath: display.item.icon },
      sounds: { ...node.sounds, show: enemy.sound ? SoundCue.Warning : SoundCue.None },
      style: {
        ...node.style,
        showTimer: options.showTimer,
        showStacks: false,
        imageSource: ImageSource.Skill,
        imageSkillId: display.item.forms[0]?.matchIds[0] ?? null,
        glow: true,
        glowColor: '#e17e63',
      },
    };

    return this.presentation(widget, options.showTimer);
  }

  private presentation(node: WidgetNode, showTimer: boolean): WidgetNode {
    if (node.kind !== WidgetKind.Bar) {
      return node;
    }

    return {
      ...node,
      appearance: {
        ...node.appearance,
        text: showTimer ? '{t1.remaining}' : '',
        fontSize: 14,
        background: '#302421',
      },
      style: {
        ...node.style,
        source: ProgressSource.Trigger,
        fillColor: '#e17e63',
        borderWidth: 0,
        barIconEnabled: true,
        barIconSize: 24,
      },
    };
  }
}
