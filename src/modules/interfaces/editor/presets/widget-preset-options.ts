import type { TriggerUnit } from '../../model/rules/trigger-unit';
import type { WidgetPresetSelection } from './widget-preset-selection';
import type { EnemyWidgetPresetOptions } from './enemy-widget-preset-options';

export interface WidgetPresetOptions {
  readonly unit: TriggerUnit.Player | TriggerUnit.Target;
  readonly showTimer: boolean;
  readonly catalog: WidgetPresetSelection | null;
  readonly enemy?: EnemyWidgetPresetOptions;
}
