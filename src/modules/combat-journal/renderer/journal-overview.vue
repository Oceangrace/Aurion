<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { JournalSegment } from '../model/journal-segment';
import type { JournalWindow } from '../model/journal-window';
import type { JournalFormatter } from '../runtime/journal-formatter';
import type { JournalQuery } from '../runtime/journal-query';
import { JournalMetric } from '../model/journal-metric';
import { JournalActionKind } from '../model/journal-action-kind';
import { JournalAnalysis } from '../runtime/journal-analysis';
import { JournalChartColor } from '../model/journal-chart-color';
import JournalTimeline from './journal-timeline.vue';

const props = defineProps<{
  segment: JournalSegment;
  window: JournalWindow;
  query: JournalQuery;
  formatter: JournalFormatter;
  timelineAvailable: boolean;
}>();

const emit = defineEmits<{
  select: [actor: number, metric: JournalMetric];
  death: [actor: number];
}>();

const { t } = useI18n();

const analysis = new JournalAnalysis();

const defeated = computed(
  () =>
    props.segment.encounterId !== null &&
    props.segment.encounterId !== undefined &&
    (props.segment.actions ?? []).some(
      (action) =>
        action.kind === JournalActionKind.Death && action.sourceId === props.segment.encounterId,
    ),
);

const metrics = [JournalMetric.Damage, JournalMetric.Healing, JournalMetric.DamageTaken];

const tables = computed(() =>
  metrics.map((metric) => ({
    metric,
    rows: props.query.rows(props.segment, {
      ...props.window,
      metric,
      limit: Number.MAX_SAFE_INTEGER,
    }),
  })),
);

const actors = computed(
  () =>
    new Set([
      ...tables.value.flatMap((table) => table.rows.map((row) => row.actorId)),
      ...deathRows.value.map((row) => row.actorId),
    ]),
);

const deathRows = computed(() =>
  props.query.rows(props.segment, {
    ...props.window,
    metric: JournalMetric.Deaths,
    limit: Number.MAX_SAFE_INTEGER,
  }),
);

const series = computed(() =>
  tables.value.map((table, index) => {
    const incoming = table.metric === JournalMetric.DamageTaken;

    const ids = new Set(table.rows.map((row) => row.actorId));

    const actions = (props.segment.actions ?? []).filter((action) => {
      const id = incoming ? action.recipientId : action.sourceId;

      return id !== null && ids.has(analysis.root(id, props.segment, props.window.mergeSummons));
    });

    return {
      label:
        t('modules.combatJournal.metrics.' + table.metric) +
        t('modules.combatJournal.analysis.perSecond'),
      color:
        [JournalChartColor.Blue, JournalChartColor.Green, JournalChartColor.Red][index] ??
        JournalChartColor.Blue,
      values: analysis.chart(
        actions,
        table.metric === JournalMetric.Healing
          ? JournalActionKind.Healing
          : JournalActionKind.Damage,
        props.segment.spanMs ?? props.segment.elapsedMs,
        props.segment.startedAtUs,
      ),
    };
  }),
);
</script>
<template>
  <div class="journal-overview">
    <div class="overview-title">
      <div>
        <h3>
          {{ formatter.npc(segment.npcId) }}
          <small v-if="defeated">{{ t('modules.combatJournal.analysis.bossDefeated') }}</small>
        </h3>
        <p class="overview-location">
          {{ formatter.zone(segment) ?? t('modules.combatJournal.analysis.unknownLocation') }}
        </p>
        <span
          >{{ formatter.time(segment.elapsedMs) }} ·
          {{ new Date(segment.startedAtUs / 1000).toLocaleString() }}</span
        >
      </div>
      <span>{{ t('modules.combatJournal.analysis.participants') }}: {{ actors.size }}</span>
    </div>
    <JournalTimeline
      v-if="timelineAvailable"
      :series="series"
      :duration="segment.spanMs ?? segment.elapsedMs"
      :formatter="formatter"
    />
    <p v-if="segment.tracePartial" class="overview-note">
      {{ t('modules.combatJournal.analysis.tracePartial') }}
    </p>
    <div class="overview-grid">
      <section v-for="table in tables" :key="table.metric" class="overview-card">
        <header>
          <h3>{{ t('modules.combatJournal.metrics.' + table.metric) }}</h3>
          <span
            >{{ table.rows.some((row) => row.value === null) ? '≥ ' : ''
            }}{{
              formatter.number(
                table.rows.reduce((sum, row) => sum + row.knownValue, 0),
                true,
              )
            }}</span
          >
        </header>
        <div class="overview-rows">
          <button
            v-for="(row, index) in table.rows"
            :key="row.id"
            type="button"
            :disabled="row.actorId === null"
            :style="{ '--actor-color': formatter.classColor(row.unit?.className) }"
            @click="row.actorId !== null && emit('select', row.actorId, table.metric)"
          >
            <span class="overview-name"
              ><small>{{ index + 1 }}</small
              ><i />{{ row.label }}</span
            ><span class="overview-amount"
              ><i :style="{ width: String(row.bar) + '%' }" /><b>{{
                formatter.value(row, true, table.metric)
              }}</b
              ><small>{{ formatter.number(row.share, false) }}%</small></span
            ><span class="overview-rate"
              >{{ row.value === null ? '≥ ' : ''
              }}{{
                segment.elapsedMs > 0
                  ? formatter.number((row.knownValue * 1000) / segment.elapsedMs, true)
                  : '—'
              }}
              /s</span
            >
          </button>
          <p v-if="table.rows.length === 0">{{ t('modules.combatJournal.analysis.noEvents') }}</p>
        </div>
      </section>
      <section class="overview-card">
        <header>
          <h3>{{ t('modules.combatJournal.analysis.tabs.deaths') }}</h3>
          <span>{{ deathRows.reduce((sum, row) => sum + row.knownValue, 0) }}</span>
        </header>
        <div class="overview-rows">
          <button
            v-for="row in deathRows"
            :key="row.id"
            type="button"
            :disabled="!timelineAvailable || row.actorId === null"
            @click="row.actorId !== null && emit('death', row.actorId)"
          >
            <span class="overview-name">{{ row.label }}</span
            ><span>{{ formatter.value(row, false, JournalMetric.Deaths) }}</span>
          </button>
          <p v-if="deathRows.length === 0">{{ t('modules.combatJournal.analysis.noDeaths') }}</p>
        </div>
      </section>
    </div>
  </div>
</template>
<style scoped>
.overview-location {
  margin: 6px 0;
  color: var(--prism-accent);
  font-size: 13px;
}
.overview-title h3 small {
  color: #7bc99f;
  font-size: 11px;
  margin-left: 10px;
}
.journal-overview {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.overview-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}
h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
}
.overview-title h3 {
  margin-bottom: 5px;
  font-size: 16px;
}
.overview-title span,
.overview-note {
  color: var(--prism-muted);
  font-size: 11px;
}
.overview-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}
.overview-card {
  border: 1px solid var(--prism-border);
  border-radius: 6px;
  overflow: hidden;
  min-width: 0;
}
.overview-card header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--prism-field);
  padding: 12px;
  border-bottom: 1px solid var(--prism-border);
  font-variant-numeric: tabular-nums;
}
.overview-card header span {
  color: var(--prism-muted);
}
.overview-rows {
  max-height: 280px;
  overflow: auto;
  padding: 4px;
}
.overview-rows button {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  border: 0;
  border-bottom: 1px solid var(--prism-border);
  background: transparent;
  color: var(--prism-text);
  padding: 7px;
  font: inherit;
  font-size: 12px;
  cursor: pointer;
  text-align: left;
}
.overview-rows button:hover {
  background: var(--prism-selected);
}
.overview-name {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.overview-name i {
  width: 4px;
  height: 20px;
  border-radius: 2px;
  background: var(--actor-color);
  flex-shrink: 0;
}
.overview-name small {
  color: var(--prism-muted);
  width: 12px;
}
.overview-amount {
  width: 40%;
  position: relative;
  display: flex;
  justify-content: space-between;
  gap: 8px;
  padding: 4px 6px;
  font-variant-numeric: tabular-nums;
}
.overview-amount > i {
  position: absolute;
  inset: 0;
  background: var(--actor-color);
  opacity: 0.22;
  border-radius: 2px;
}
.overview-amount b,
.overview-amount small {
  position: relative;
  font-weight: 500;
}
.overview-rate {
  width: 70px;
  text-align: right;
  color: var(--prism-muted);
  font-size: 11px;
  white-space: nowrap;
}
.overview-rows p {
  color: var(--prism-muted);
  font-size: 12px;
  padding: 12px;
}
@media (max-width: 850px) {
  .overview-grid {
    grid-template-columns: 1fr;
  }
}
</style>
