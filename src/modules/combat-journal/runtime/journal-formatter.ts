import type { JournalRow } from '../model/journal-row';
import type { JournalSegment } from '../model/journal-segment';
import type { JournalAppearance } from '../model/journal-appearance';
import { ClassPalette } from '../../../core/preferences/class-palette';
import type { CharacterClass } from '../../../core/preferences/character-class';
import type { EffectKind } from '../../../core/catalog/effect-kind';
import { JournalMetric } from '../model/journal-metric';
import type { CatalogLabels } from '../../../core/catalog/catalog-labels';
export class JournalFormatter {
  private readonly colors = new ClassPalette();

  public constructor(
    private readonly locale: () => string,
    private readonly labels: () => CatalogLabels,
  ) {}

  public number(value: number, compact: boolean): string {
    return new Intl.NumberFormat(this.locale(), {
      notation: compact ? 'compact' : 'standard',
      maximumFractionDigits: 1,
    }).format(value);
  }

  public amount(value: number | null): string {
    return value === null ? '—' : this.number(value, false);
  }

  public percent(value: number | null): string {
    return value === null ? '—' : this.amount(value) + '%';
  }

  public value(row: JournalRow, compact: boolean, metric: JournalMetric): string {
    if (row.value === null && metric === JournalMetric.Critical) {
      return '—';
    }

    return (row.value === null ? '≥ ' : '') + this.number(row.knownValue, compact);
  }

  public skill(id: number | null): string {
    return id === null ? '—' : (this.labels().skills[id] ?? String(id));
  }

  public npc(id: number | null): string {
    return id === null ? '—' : (this.labels().npcs[id] ?? 'NPC ' + String(id));
  }

  public zone(segment: JournalSegment): string | null {
    return this.labels().zones?.[segment.mapId ?? 0] ?? null;
  }

  public unit(id: number, segment: JournalSegment): string {
    const unit = segment.units[id];

    return unit?.name ?? this.npc(unit?.npcId ?? null);
  }

  public icon(skill: number | null, effect: number | null = null): string | null {
    const labels = this.labels();

    return labels.effectIcons?.[effect ?? 0] ?? labels.skillIcons?.[skill ?? 0] ?? null;
  }

  public effect(effect: number | null, skill: number | null): string {
    return (effect === null ? undefined : this.labels().effects?.[effect]) ?? this.skill(skill);
  }

  public effectKind(effect: number | null): EffectKind | null {
    return effect === null ? null : (this.labels().effectKinds?.[effect] ?? null);
  }

  public effectVisible(effect: number | null): boolean {
    return effect === null || this.labels().hiddenEffects?.includes(effect) !== true;
  }

  public classColor(name: CharacterClass | null | undefined): string {
    return this.colors.color(name ?? null, 'var(--prism-accent)');
  }

  public time(milliseconds: number): string {
    const seconds = Math.floor(milliseconds / 1000);

    return String(Math.floor(seconds / 60)) + ':' + String(seconds % 60).padStart(2, '0');
  }

  public color(row: JournalRow, appearance: JournalAppearance): string {
    return appearance.classColors
      ? this.colors.color(row.unit?.className, appearance.accent)
      : appearance.accent;
  }
}
