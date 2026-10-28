import type { PreviewVisual } from '../../model/preview-visual';
import type { WidgetNode } from '../../model/widget-node';
import { WidgetKind } from '../../model/widget-kind';
import { SoundCue } from '../../model/sound-cue';
import { WidgetPreset } from './widget-preset';
import { ProgressSource } from '../../model/progress-source';

export class WidgetPresetPreview {
  private readonly remaining: Partial<Readonly<Record<WidgetPreset, number>>> = {
    [WidgetPreset.EnemyCast]: 3.5,
    [WidgetPreset.EventCountdown]: 5,
  };

  private readonly texts: Partial<Readonly<Record<WidgetPreset, string>>> = {
    [WidgetPreset.Health]: '18 000 / 25 000',
    [WidgetPreset.Mana]: '7 200 / 10 000',
    [WidgetPreset.Stamina]: '72 000',
    [WidgetPreset.Cast]: '1.8',
    [WidgetPreset.HealthPercent]: '72%',
    [WidgetPreset.Name]: 'Aurion',
    [WidgetPreset.CastTimer]: '1.8',
  };

  public widget(node: WidgetNode, preset: WidgetPreset): WidgetNode {
    return {
      ...node,
      appearance: {
        ...node.appearance,
        text: this.text(node, preset),
      },
      style: { ...node.style, barValue: 72 },
    };
  }

  public visual(node: WidgetNode, preset: WidgetPreset): PreviewVisual {
    const timer =
      node.kind === WidgetKind.Icon &&
      preset !== WidgetPreset.Empty &&
      preset !== WidgetPreset.Ready;

    return {
      className: null,
      simulating: true,
      animationValue: null,
      active: true,
      visible: true,
      remaining: timer ? (this.remaining[preset] ?? 18) : null,
      duration: timer ? 30 : null,
      stacks: null,
      knownProgress: true,
      progressIdentity: null,
      healingPrediction: null,
      ticks: [],
      animationActive: false,
      sound: SoundCue.None,
      speechText: '',
      offsetX: 0,
      offsetY: 0,
      childVisuals: {},
      instances: [],
      appearance: node.appearance,
      style: node.style,
    };
  }

  private text(node: WidgetNode, preset: WidgetPreset): string {
    if (
      node.kind === WidgetKind.Bar &&
      node.style.source === ProgressSource.Trigger &&
      node.appearance.text !== ''
    ) {
      return String(this.remaining[preset] ?? 18);
    }

    return this.texts[preset] ?? node.appearance.text;
  }
}
