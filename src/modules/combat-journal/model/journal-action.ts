import type { JournalActionKind } from './journal-action-kind';
export interface JournalAction {
  readonly sequence: number;
  readonly atUs: number;
  readonly kind: JournalActionKind;
  readonly sourceId: number;
  readonly recipientId: number | null;
  readonly skillId: number | null;
  readonly value: number;
  readonly durationMs?: number | null;
  readonly skillEffectId?: number | null;
}
