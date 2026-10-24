<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { JournalAnalysisRow } from '../model/journal-analysis-row';
import type { JournalFormatter } from '../runtime/journal-formatter';
import { JournalAnalysis } from '../runtime/journal-analysis';
import { JournalAnalysisTab } from '../model/journal-analysis-tab';
import { CombatCounter } from '../model/combat-counter';
import UiButton from '../../../renderer/components/ui/ui-button.vue';
import JournalSkillIcon from './journal-skill-icon.vue';
const props = defineProps<{
  entries: readonly JournalAnalysisRow[];
  skill: string | null;
  healing: boolean;
  timelineAvailable: boolean;
  castCount: (id: string) => number | null;
  formatter: JournalFormatter;
}>();

const emit = defineEmits<{ skill: [id: string | null]; tab: [tab: JournalAnalysisTab] }>();

const { t } = useI18n();

const analysis = new JournalAnalysis();

const criticalCounters = [
  CombatCounter.CriticalHits,
  CombatCounter.PerfectHits,
  CombatCounter.DoubleHits,
  CombatCounter.BackHits,
  CombatCounter.FrontalHits,
  CombatCounter.ParryHits,
  CombatCounter.ShieldBlockHits,
  CombatCounter.AdditionalHitEvents,
];

const selected = computed(
  () => props.entries.find((entry) => entry.row.id === props.skill) ?? null,
);

const flags = computed(() =>
  criticalCounters.map((counter) => ({
    counter,
    value: selected.value === null ? null : analysis.ratio(selected.value.row, counter),
  })),
);

const otherCounters = computed(() =>
  Object.values(CombatCounter)
    .map((counter) => ({ counter, value: selected.value?.row.statistics.counters[counter] ?? 0 }))
    .filter((entry) => entry.value !== 0),
);
</script>
<template>
  <table class="analysis-table">
    <thead>
      <tr>
        <th>{{ t('modules.combatJournal.analysis.skill') }}</th>
        <th>{{ t('modules.combatJournal.analysis.amount') }}</th>
        <th>{{ t('modules.combatJournal.analysis.hitEvents') }}</th>
        <th>{{ t('modules.combatJournal.analysis.confirmedCasts') }}</th>
        <th>{{ t('modules.combatJournal.analysis.average') }}</th>
        <th>{{ t('modules.combatJournal.analysis.critical') }}</th>
        <th>{{ t('modules.combatJournal.analysis.periodic') }}</th>
      </tr>
    </thead>
    <tbody>
      <template v-for="entry in entries" :key="entry.row.id"
        ><tr :class="{ 'analysis-selected': skill === entry.row.id }">
          <td>
            <button
              type="button"
              @click="emit('skill', skill === entry.row.id ? null : entry.row.id)"
            >
              <JournalSkillIcon :path="formatter.icon(Number(entry.row.id))" />
              {{ entry.row.label }}
            </button>
            <div class="analysis-share">
              <i :style="{ width: String(entry.row.share) + '%' }" />
            </div>
          </td>
          <td>
            {{ entry.row.value === null ? '≥ ' : '' }}{{ formatter.amount(entry.amount)
            }}<small
              >{{ formatter.percent(entry.row.share) }} · {{ formatter.amount(entry.rate)
              }}{{ t('modules.combatJournal.analysis.perSecond') }}</small
            >
          </td>
          <td>{{ formatter.amount(entry.hits) }}</td>
          <td>{{ formatter.amount(castCount(entry.row.id)) }}</td>
          <td>{{ formatter.amount(entry.average) }}</td>
          <td>{{ healing ? '—' : formatter.percent(entry.critical) }}</td>
          <td>{{ formatter.amount(entry.periodic) }}</td>
        </tr>
        <tr v-if="selected && skill === entry.row.id" class="analysis-detail-row">
          <td colspan="7">
            <section class="analysis-breakdown">
              <h3>{{ selected.row.label }}</h3>
              <div class="analysis-drilldown">
                <UiButton
                  v-if="timelineAvailable"
                  @click="emit('tab', JournalAnalysisTab.Events)"
                  >{{ t('modules.combatJournal.analysis.tabs.events') }}</UiButton
                ><UiButton
                  v-if="timelineAvailable"
                  @click="emit('tab', JournalAnalysisTab.Rotation)"
                  >{{ t('modules.combatJournal.analysis.tabs.rotation') }}</UiButton
                ><UiButton @click="emit('skill', null)">{{
                  t('modules.combatJournal.analysis.allSkills')
                }}</UiButton>
              </div>
              <p v-if="!healing" class="analysis-note">
                {{ t('modules.combatJournal.analysis.directDenominator') }}
              </p>
              <div v-if="!healing" class="analysis-flags">
                <div v-for="flag in flags" :key="flag.counter">
                  <span>{{ t('modules.combatJournal.counters.' + flag.counter) }}</span
                  ><b>{{ formatter.percent(flag.value) }}</b>
                </div>
              </div>
              <dl>
                <template v-if="!healing">
                  <dt>{{ t('modules.combatJournal.analysis.minimum') }}</dt>
                  <dd>{{ formatter.amount(selected.row.statistics.minimum) }}</dd>
                  <dt>{{ t('modules.combatJournal.analysis.maximum') }}</dt>
                  <dd>{{ formatter.amount(selected.row.statistics.maximum) }}</dd>
                </template>
                <template v-for="counter in otherCounters" :key="counter.counter"
                  ><dt>{{ t('modules.combatJournal.counters.' + counter.counter) }}</dt>
                  <dd>{{ formatter.amount(counter.value) }}</dd></template
                >
              </dl>
            </section>
          </td>
        </tr></template
      >
    </tbody>
  </table>
</template>
<style scoped src="./journal-analysis-table.css"></style>
<style scoped>
.analysis-table td small {
  display: block;
  color: var(--prism-muted);
  font-size: 11px;
  margin-top: 5px;
}
.analysis-table button {
  display: flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: 0;
  color: inherit;
  text-align: left;
  padding: 0;
  cursor: pointer;
  font: inherit;
}
.analysis-share {
  height: 3px;
  background: var(--prism-field);
  margin-top: 7px;
}
.analysis-share i {
  display: block;
  background: var(--prism-accent);
  height: 100%;
}
.analysis-selected {
  background: var(--prism-selected);
}
.analysis-breakdown {
  text-align: left;
  white-space: normal;
  padding: 8px 12px;
}
.analysis-breakdown h3 {
  font-size: 14px;
}
.analysis-drilldown {
  display: flex;
  align-items: center;
  gap: 12px;
}
.analysis-note {
  font-size: 12px;
  line-height: 1.5;
  color: var(--prism-muted);
  margin: 0 0 12px;
}
.analysis-flags {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  padding: 12px 0;
}
.analysis-flags > div {
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.analysis-flags span {
  color: var(--prism-muted);
  font-size: 12px;
}
.analysis-breakdown dl {
  display: grid;
  grid-template-columns: 1fr auto 1fr auto;
  gap: 8px;
}
.analysis-breakdown dd {
  margin: 0 20px 0 0;
  font-variant-numeric: tabular-nums;
}
@media (max-width: 650px) {
  .analysis-flags {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
