import type { WidgetPresetSelection } from './widget-preset-selection';

export interface EnemyWidgetPresetOptions {
  readonly actorKey: string | null;
  readonly skill: WidgetPresetSelection | null;
  readonly expectedSkill: WidgetPresetSelection | null;
  readonly countdownSeconds: number;
  readonly warningSeconds: number;
  readonly sound: boolean;
}

export const defaultEnemyWidgetPresetOptions: EnemyWidgetPresetOptions = {
  actorKey: null,
  skill: null,
  expectedSkill: null,
  countdownSeconds: 25,
  warningSeconds: 5,
  sound: false,
};
