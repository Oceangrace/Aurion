import type { PreviewVisual } from './preview-visual';
import type { SourceStatus } from './source-status';
import type { TriggerState } from './trigger-state';

export interface LiveFrame {
  readonly status: SourceStatus;
  readonly inWorld: boolean;
  readonly characterDataComplete: boolean;
  readonly atUs: number | null;
  readonly error: string | null;
  readonly visuals: Readonly<Record<string, PreviewVisual>>;
  readonly states: Readonly<Record<string, TriggerState>>;
  readonly documentErrors: Readonly<Record<string, string>>;
}
