<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { JournalSegment } from '../model/journal-segment';
import type { JournalWindow } from '../model/journal-window';
import type { JournalAnalysisRow } from '../model/journal-analysis-row';
import type { JournalQuery } from '../runtime/journal-query';
import type { JournalFormatter } from '../runtime/journal-formatter';
import { JournalAnalysis } from '../runtime/journal-analysis';
import { JournalDetail } from '../model/journal-detail';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import JournalSkillIcon from './journal-skill-icon.vue';
const props = defineProps<{
  segment: JournalSegment;
  segments: readonly JournalSegment[];
  entries: readonly JournalAnalysisRow[];
  window: JournalWindow;
  query: JournalQuery;
  formatter: JournalFormatter;
  search: string;
  actorLabel: string | null;
  reference: string;
  referenceActor: string;
}>();

const emit = defineEmits<{
  reference: [fight: string, actor: string];
  referenceActor: [actor: string];
}>();

const { t } = useI18n();

const analysis = new JournalAnalysis();

const options = computed(() => [
  { value: 'none', label: t('modules.combatJournal.analysis.chooseFight') },
  ...props.segments
    .filter((entry) => entry.id !== props.segment.id && entry.endedAtUs !== null)
    .map((entry) => ({
      value: entry.id,
      label:
        props.formatter.npc(entry.npcId) +
        ' · ' +
        props.formatter.time(entry.elapsedMs) +
        ' · ' +
        new Date(entry.startedAtUs / 1000).toLocaleString(),
    })),
]);

const fight = (id: string): JournalSegment | null =>
  props.segments.find((entry) => entry.id === id) ?? null;

const other = computed(() => fight(props.reference));

const actorOptions = computed(() =>
  props.query
    .rows(other.value, props.window)
    .map((row) => ({ value: String(row.actorId), label: row.label })),
);

/** Picks the same participant by name in the reference fight, else its player. */
const choose = (id: string): void => {
  const segment = fight(id);

  const rows = props.query.rows(segment, props.window);

  const same = rows.find((row) => row.label === props.actorLabel);

  emit('reference', id, String(same?.actorId ?? segment?.playerId ?? rows[0]?.actorId ?? ''));
};

const compared = computed(() => {
  const current = new Map(props.entries.map((entry) => [entry.row.id, entry]));

  const previous = new Map(
    props.query
      .rows(other.value, props.window, JournalDetail.Skills, Number(props.referenceActor))
      .map((row) => analysis.row(row, props.window.metric, other.value?.elapsedMs ?? 0))
      .map((entry) => [entry.row.id, entry]),
  );

  const search = props.search.toLocaleLowerCase();

  return [...new Set([...current.keys(), ...previous.keys()])]
    .map((id) => ({ id, current: current.get(id), previous: previous.get(id) }))
    .filter((entry) =>
      (entry.current?.row.label ?? entry.previous?.row.label ?? '')
        .toLocaleLowerCase()
        .includes(search),
    );
});

const delta = (
  current: JournalAnalysisRow | undefined,
  previous: JournalAnalysisRow | undefined,
): string => {
  const value = analysis.difference(current, previous);

  return value === null ? '—' : (value > 0 ? '+' : '') + props.formatter.amount(value);
};
</script>
<template>
  <div class="analysis-filters">
    <UiSelect
      :label="t('modules.combatJournal.analysis.reference')"
      :model-value="reference"
      :options="options"
      @update:model-value="choose($event)"
    /><UiSelect
      v-if="other"
      :label="t('modules.combatJournal.analysis.referenceActor')"
      :model-value="referenceActor"
      :options="actorOptions"
      @update:model-value="emit('referenceActor', $event)"
    />
  </div>
  <table v-if="other" class="analysis-table">
    <thead>
      <tr>
        <th>{{ t('modules.combatJournal.analysis.skill') }}</th>
        <th>
          {{ t('modules.combatJournal.analysis.current')
          }}{{ t('modules.combatJournal.analysis.perSecond') }}
        </th>
        <th>
          {{ t('modules.combatJournal.analysis.reference')
          }}{{ t('modules.combatJournal.analysis.perSecond') }}
        </th>
        <th>Δ {{ t('modules.combatJournal.analysis.perSecond') }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="entry in compared" :key="entry.id">
        <td>
          <span class="analysis-skill-name"
            ><JournalSkillIcon :path="formatter.icon(Number(entry.id))" />{{
              entry.current?.row.label ?? entry.previous?.row.label
            }}</span
          >
        </td>
        <td>{{ formatter.amount(entry.current?.rate ?? 0) }}</td>
        <td>{{ formatter.amount(entry.previous?.rate ?? 0) }}</td>
        <td>{{ delta(entry.current, entry.previous) }}</td>
      </tr>
    </tbody>
  </table>
</template>
<style scoped src="./journal-analysis-table.css"></style>
<style scoped>
.analysis-filters {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  padding: 0 0 14px;
}
.analysis-skill-name {
  display: flex;
  gap: 8px;
  align-items: center;
}
@media (max-width: 650px) {
  .analysis-filters {
    grid-template-columns: 1fr;
  }
}
</style>
