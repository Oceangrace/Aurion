<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { JournalSegment } from '../model/journal-segment';
import type { JournalWindow } from '../model/journal-window';
import type { JournalQuery } from '../runtime/journal-query';
import type { JournalFormatter } from '../runtime/journal-formatter';
import type { JournalAction } from '../model/journal-action';
import { JournalAnalysis } from '../runtime/journal-analysis';
import { JournalAnalysisTab } from '../model/journal-analysis-tab';
import { JournalAnalysisSort } from '../model/journal-analysis-sort';
import { JournalActionKind } from '../model/journal-action-kind';
import { JournalChartColor } from '../model/journal-chart-color';
import { JournalDetail } from '../model/journal-detail';
import { JournalMetric } from '../model/journal-metric';
import JournalComparePanel from './journal-compare-panel.vue';
import JournalDeathsPanel from './journal-deaths-panel.vue';
import JournalEffectsPanel from './journal-effects-panel.vue';
import JournalOverview from './journal-overview.vue';
import JournalSkillsTable from './journal-skills-table.vue';
import JournalTimeline from './journal-timeline.vue';
import JournalEventsPanel from './journal-events-panel.vue';
import JournalCastsPanel from './journal-casts-panel.vue';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import { JournalScope } from '../model/journal-scope';
import UiSwitch from '../../../renderer/components/ui/ui-switch.vue';
import JournalFightPicker from './journal-fight-picker.vue';
import JournalIntervalPanel from './journal-interval-panel.vue';
import UiButton from '../../../renderer/components/ui/ui-button.vue';
import UiIconButton from '../../../renderer/components/ui/ui-icon-button.vue';
import { JournalNavigation } from '../runtime/journal-navigation';
import type { JournalNavigationState } from '../model/journal-navigation-state';

const props = defineProps<{
  segment: JournalSegment;
  segments: readonly JournalSegment[];
  actor: number;
  window: JournalWindow;
  query: JournalQuery;
  formatter: JournalFormatter;
  exportFile: (text: string) => Promise<void>;
}>();

const { t } = useI18n();

const analysis = new JournalAnalysis();

/** Everything Back/Forward restores. Child panels read it and report changes back. */
const view = reactive<{ -readonly [K in keyof JournalNavigationState]: JournalNavigationState[K] }>(
  {
    fight: null,
    tab: JournalAnalysisTab.Overview,
    actor: props.actor,
    metric:
      new Map([
        [JournalMetric.DamageTaken, JournalMetric.DamageTaken],
        [JournalMetric.HealingTaken, JournalMetric.HealingTaken],
        [JournalMetric.Healing, JournalMetric.Healing],
        [JournalMetric.Hps, JournalMetric.Healing],
      ]).get(props.window.metric) ?? JournalMetric.Damage,
    scope: props.window.scope,
    mergeSummons: props.window.mergeSummons,
    target: 'all',
    skill: null,
    reference: 'none',
    referenceActor: '',
    bySkill: false,
    raidCasts: true,
    sort: JournalAnalysisSort.Amount,
    search: '',
    interval: [0, props.segment.spanMs ?? props.segment.elapsedMs],
  },
);

/** Set while Back/Forward applies a state, so the resets below do not overwrite it. */
let restoring = false;

const segment = computed(
  () => props.segments.find((fight) => fight.id === view.fight) ?? props.segment,
);

const duration = computed(() => segment.value.spanMs ?? segment.value.elapsedMs);

const timelineAvailable = computed(() => (segment.value.actions?.length ?? 0) !== 0);

const effectsAvailable = computed(() => (segment.value.effects?.length ?? 0) !== 0);

const quantitative = computed(() =>
  [
    JournalAnalysisTab.Skills,
    JournalAnalysisTab.Targets,
    JournalAnalysisTab.Timeline,
    JournalAnalysisTab.Events,
    JournalAnalysisTab.Compare,
  ].includes(view.tab),
);

const scroll = ref<HTMLElement | null>(null);

const exportError = ref('');

const castActor = (id: number): number => analysis.root(id, segment.value, view.mergeSummons);

const castActors = computed(() => [
  ...new Set(props.query.participants(segment.value, view.scope).map(castActor)),
]);

const groupActions = computed(() => {
  const ids = new Set(castActors.value);

  return (segment.value.actions ?? []).filter((action) => ids.has(castActor(action.sourceId)));
});

const analysisWindow = computed(() => ({
  ...props.window,
  metric: view.metric,
  scope: view.scope,
  mergeSummons: view.mergeSummons,
  limit: Number.MAX_SAFE_INTEGER,
}));

const metricOptions = computed(() =>
  [
    JournalMetric.Damage,
    JournalMetric.Healing,
    JournalMetric.DamageTaken,
    JournalMetric.HealingTaken,
  ].map((value) => ({ value, label: t('modules.combatJournal.metrics.' + value) })),
);

const actorOptions = computed(() =>
  [
    ...new Set([
      view.actor,
      ...(segment.value.actions ?? []).map((action) =>
        analysis.root(action.sourceId, segment.value, view.mergeSummons),
      ),
      ...props.query
        .rows(segment.value, { ...analysisWindow.value, scope: JournalScope.All })
        .flatMap((row) => (row.actorId === null ? [] : [row.actorId])),
    ]),
  ].map((id) => ({ value: String(id), label: props.formatter.unit(id, segment.value) })),
);

const selectActor = (id: number, value: JournalMetric): void => {
  view.actor = id;
  view.metric = value;
  view.tab = JournalAnalysisTab.Skills;
};

const selectDeath = (id: number): void => {
  selectActor(id, JournalMetric.DamageTaken);
  view.tab = JournalAnalysisTab.Deaths;
};

const availableTabs = computed<Readonly<Record<JournalAnalysisTab, boolean>>>(() => ({
  [JournalAnalysisTab.Overview]: true,
  [JournalAnalysisTab.Skills]: true,
  [JournalAnalysisTab.Targets]: true,
  [JournalAnalysisTab.Compare]: true,
  [JournalAnalysisTab.Effects]: effectsAvailable.value,
  [JournalAnalysisTab.Timeline]: timelineAvailable.value,
  [JournalAnalysisTab.Rotation]: timelineAvailable.value,
  [JournalAnalysisTab.Deaths]: timelineAvailable.value,
  [JournalAnalysisTab.Events]: timelineAvailable.value,
}));

watch([availableTabs, () => view.tab], ([available, mode]) => {
  if (available[mode] === false) {
    view.tab = JournalAnalysisTab.Skills;
  }
});

const incoming = computed(
  () => view.metric === JournalMetric.DamageTaken || view.metric === JournalMetric.HealingTaken,
);

const healing = computed(() => analysis.healing(view.metric));

const amountMetric = computed(() => (healing.value ? JournalMetric.Healing : JournalMetric.Damage));

const chooseActor = (id: number): void => {
  if (
    props.query.rows(segment.value, analysisWindow.value).some((row) => row.actorId === id) ===
    false
  ) {
    view.scope = JournalScope.All;
  }

  view.actor = id;
};

const targetRows = computed(() =>
  props.query.rows(segment.value, analysisWindow.value, JournalDetail.Targets, view.actor),
);

const targetOptions = computed(() => [
  { value: 'all', label: t('modules.combatJournal.analysis.allTargets') },
  ...targetRows.value.map((row) => ({ value: row.id, label: row.label })),
]);

const targetId = computed(() => (view.target === 'all' ? null : Number(view.target)));

const filtered = computed<JournalSegment>(() => ({
  ...segment.value,
  records: segment.value.records.filter(
    (record) =>
      view.target === 'all' ||
      String(incoming.value ? record.sourceId : record.recipientId) === view.target,
  ),
}));

const actorRow = computed(
  () =>
    props.query
      .rows(filtered.value, analysisWindow.value)
      .find((row) => row.actorId === view.actor) ?? null,
);

const summary = computed(() =>
  actorRow.value === null
    ? null
    : analysis.row(actorRow.value, view.metric, segment.value.elapsedMs),
);

const entries = computed(() =>
  props.query
    .rows(filtered.value, analysisWindow.value, JournalDetail.Skills, view.actor)
    .map((row) => analysis.row(row, view.metric, segment.value.elapsedMs)),
);

const visible = computed(() =>
  analysis.ordered(
    entries.value.filter((entry) =>
      entry.row.label.toLocaleLowerCase().includes(view.search.toLocaleLowerCase()),
    ),
    view.sort,
  ),
);

const selected = computed(() => entries.value.find((entry) => entry.row.id === view.skill) ?? null);

const actions = computed(() =>
  analysis.actions(segment.value, view.actor, view.mergeSummons, incoming.value, targetId.value),
);

const full = computed(
  () => segment.value.tracePartial !== true && (segment.value.actions?.length ?? 0) !== 0,
);

const castCounts = computed(() => {
  const counts = new Map<string, number>();

  for (const action of actions.value) {
    if (action.kind === JournalActionKind.Cast) {
      const key = String(action.skillId);

      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }

  return counts;
});

const castCount = (id: string): number | null =>
  full.value && view.target === 'all' && !incoming.value ? (castCounts.value.get(id) ?? 0) : null;

const bySkill = (action: JournalAction): boolean =>
  view.skill === null || String(action.skillId) === view.skill;

const eventActions = computed(() => actions.value.filter(bySkill));

const rotationActions = computed(() =>
  analysis
    .actions(segment.value, view.actor, view.mergeSummons, false, targetId.value)
    .filter(bySkill),
);

const timelineEvents = computed(() =>
  eventActions.value.filter((action) => {
    const at = (action.atUs - segment.value.startedAtUs) / 1000;

    return (
      at >= view.interval[0] &&
      (at < view.interval[1] || (view.interval[1] >= duration.value && at === view.interval[1]))
    );
  }),
);

const series = (values: readonly JournalAction[], kind: JournalActionKind): readonly number[] =>
  analysis.chart(values, kind, duration.value, segment.value.startedAtUs);

const palette = [
  JournalChartColor.Blue,
  JournalChartColor.Violet,
  JournalChartColor.Green,
  JournalChartColor.Amber,
  JournalChartColor.Red,
  JournalChartColor.Grey,
];

const abilitySeries = computed(() => {
  const top = entries.value.slice(0, 5);

  const ids = new Set(top.map((entry) => entry.row.id));

  const groups = [
    ...top.map((entry) => ({ id: entry.row.id, label: entry.row.label })),
    { id: 'rest', label: t('modules.combatJournal.analysis.otherSkills') },
  ];

  return groups.map((group, index) => ({
    label: group.label,
    color: palette[index] ?? JournalChartColor.Grey,
    values: series(
      eventActions.value.filter((action) =>
        group.id === 'rest'
          ? !ids.has(String(action.skillId))
          : String(action.skillId) === group.id,
      ),
      healing.value ? JournalActionKind.Healing : JournalActionKind.Damage,
    ),
  }));
});

const chartSeries = computed(() => [
  {
    label:
      t(
        'modules.combatJournal.metrics.' +
          (incoming.value ? JournalMetric.DamageTaken : JournalMetric.Damage),
      ) + '/s',
    color: JournalChartColor.Blue,
    values: series(eventActions.value, JournalActionKind.Damage),
  },
  {
    label:
      t(
        'modules.combatJournal.metrics.' +
          (incoming.value ? JournalMetric.HealingTaken : JournalMetric.Healing),
      ) + '/s',
    color: JournalChartColor.Green,
    values: series(eventActions.value, JournalActionKind.Healing),
  },
]);

const chartKey = computed(
  () =>
    segment.value.id +
    ':' +
    String(view.actor) +
    ':' +
    view.metric +
    ':' +
    view.target +
    ':' +
    String(view.skill),
);

const clearFilters = (): void => {
  view.target = 'all';
  view.skill = null;
};

watch(
  [() => segment.value.id, () => view.actor, () => view.metric, () => view.mergeSummons],
  () => {
    if (restoring === true) {
      return;
    }

    clearFilters();
    view.reference = 'none';
    view.referenceActor = '';
    view.interval = [0, duration.value];
  },
);
watch(
  () => props.actor,
  (value) => {
    view.actor = value;
  },
);
watch(
  () => segment.value.id,
  () => {
    if (restoring === true) {
      return;
    }

    view.actor = segment.value.playerId ?? props.actor;
    view.tab = JournalAnalysisTab.Overview;
  },
);
watch(
  () => view.tab,
  (value) => {
    if (restoring === false && value === JournalAnalysisTab.Compare) {
      clearFilters();
    }
  },
);
watch(
  () => view.raidCasts,
  (value) => {
    if (restoring === false && value === true) {
      clearFilters();
    }
  },
);
watch(
  [() => view.tab, () => view.actor, () => view.metric, () => view.target],
  () => {
    scroll.value?.scrollTo({ top: 0, left: 0 });
  },
  { flush: 'post' },
);

const exportCsv = async (): Promise<void> => {
  try {
    const header = [
      'skill',
      'amount',
      'share_percent',
      'rate',
      'hit_events',
      'average_event',
      'critical_direct_percent',
      'periodic_amount',
      'complete',
      'activations',
    ];

    const values = visible.value.map((entry) => [
      ...analysis.exportRow(entry),
      castCount(entry.row.id) ?? '',
    ]);

    await props.exportFile(analysis.csv([header, ...values]));
    exportError.value = '';
  } catch (error) {
    exportError.value = String(error);
  }
};

const location = (): JournalNavigationState => ({ ...view });

const navigation = reactive(new JournalNavigation(location()));

const restore = async (state: JournalNavigationState | null): Promise<void> => {
  if (state === null) {
    return;
  }

  restoring = true;
  Object.assign(view, state);
  await nextTick();
  restoring = false;
};

/** A change of these opens a new history entry; the rest only refine the current one. */
const visited = [
  'fight',
  'tab',
  'actor',
  'metric',
  'scope',
  'mergeSummons',
  'target',
  'skill',
  'reference',
  'referenceActor',
  'raidCasts',
] as const;

const refined = ['search', 'sort', 'bySkill', 'interval'] as const;

watch(
  visited.map((key) => () => view[key]),
  () => {
    if (restoring === false) {
      navigation.visit(location());
    }
  },
  { flush: 'post' },
);
watch(
  refined.map((key) => () => view[key]),
  () => {
    if (restoring === false) {
      navigation.update(location());
    }
  },
  { flush: 'post' },
);

const navigateKey = (event: KeyboardEvent): void => {
  if (event.altKey === false || ['ArrowLeft', 'ArrowRight'].includes(event.key) === false) {
    return;
  }

  event.preventDefault();
  void restore(event.key === 'ArrowLeft' ? navigation.back() : navigation.forward());
};
</script>
<template>
  <div class="journal-analysis" @keydown="navigateKey">
    <div class="analysis-toolbar">
      <div class="analysis-history">
        <UiIconButton
          :label="t('modules.combatJournal.analysis.back') + ' (Alt+←)'"
          :disabled="!navigation.canBack"
          @click="restore(navigation.back())"
        >
          <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m11 4-6 6 6 6M5 10h11" /></svg>
        </UiIconButton>
        <UiIconButton
          :label="t('modules.combatJournal.analysis.forward') + ' (Alt+→)'"
          :disabled="!navigation.canForward"
          @click="restore(navigation.forward())"
        >
          <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m9 4 6 6-6 6M4 10h11" /></svg>
        </UiIconButton>
      </div>
      <JournalFightPicker
        :segment="segment"
        :segments="segments"
        :formatter="formatter"
        @select="view.fight = $event"
      />
    </div>
    <div class="analysis-filters analysis-scope">
      <UiSelect
        :label="t('modules.combatJournal.analysis.participants')"
        :model-value="view.scope"
        :options="
          Object.values(JournalScope).map((value) => ({
            value,
            label: t('modules.combatJournal.scopes.' + value),
          }))
        "
        @update:model-value="view.scope = $event"
      />
      <UiSwitch v-model="view.mergeSummons" :label="t('modules.combatJournal.summons')" />
    </div>
    <nav class="analysis-tabs" :aria-label="t('modules.combatJournal.analysis.navigation')">
      <button
        v-for="mode in Object.values(JournalAnalysisTab)"
        :key="mode"
        type="button"
        :aria-pressed="view.tab === mode"
        :title="
          !availableTabs[mode] ? t('modules.combatJournal.analysis.traceUnavailable') : undefined
        "
        :disabled="!availableTabs[mode]"
        @click="view.tab = mode"
      >
        {{ t('modules.combatJournal.analysis.tabs.' + mode) }}
      </button>
    </nav>
    <div
      ref="scroll"
      class="analysis-scroll"
      :class="{ 'analysis-scroll-rotation': view.tab === JournalAnalysisTab.Rotation }"
    >
      <p v-if="healing" class="analysis-note">
        {{ t('modules.combatJournal.analysis.healingAmountHint') }}
      </p>
      <div
        v-if="view.tab === JournalAnalysisTab.Rotation"
        class="analysis-chart-modes cast-view-modes"
      >
        <button type="button" :aria-pressed="view.raidCasts" @click="view.raidCasts = true">
          {{ t('modules.combatJournal.analysis.allParticipants') }}
        </button>
        <button type="button" :aria-pressed="!view.raidCasts" @click="view.raidCasts = false">
          {{ t('modules.combatJournal.analysis.selectedParticipant') }}
        </button>
      </div>
      <header
        v-if="
          view.tab !== JournalAnalysisTab.Overview &&
          !(view.tab === JournalAnalysisTab.Rotation && view.raidCasts)
        "
        class="analysis-context"
      >
        <div>
          <strong>{{ actorRow?.label ?? formatter.unit(view.actor, segment) }}</strong
          ><small
            >{{ formatter.time(segment.elapsedMs) }} ·
            {{ t('modules.combatJournal.metrics.' + view.metric) }}</small
          >
        </div>
        <UiButton
          v-if="view.tab === JournalAnalysisTab.Skills || view.tab === JournalAnalysisTab.Compare"
          @click="exportCsv"
          >CSV</UiButton
        >
      </header>
      <div
        v-if="
          view.tab !== JournalAnalysisTab.Overview &&
          !(view.tab === JournalAnalysisTab.Rotation && view.raidCasts)
        "
        class="analysis-filters analysis-selector"
      >
        <UiSelect
          :label="t('modules.combatJournal.analysis.participants')"
          :model-value="String(view.actor)"
          :options="actorOptions"
          @update:model-value="chooseActor(Number($event))"
        />
        <UiSelect
          v-if="quantitative"
          :label="t('modules.combatJournal.metric')"
          :model-value="view.metric"
          :options="metricOptions"
          @update:model-value="view.metric = $event"
        />
      </div>
      <div v-if="summary && quantitative" class="analysis-summary">
        <div>
          <span>{{ t('modules.combatJournal.metrics.' + amountMetric) }}</span
          ><b>{{ summary.row.value === null ? '≥ ' : '' }}{{ formatter.amount(summary.amount) }}</b>
        </div>
        <div>
          <span>{{ healing ? 'HPS' : 'DPS' }}</span
          ><b>{{ summary.row.value === null ? '≥ ' : '' }}{{ formatter.amount(summary.rate) }}</b>
        </div>
        <div>
          <span>{{ t('modules.combatJournal.analysis.share') }}</span
          ><b>{{ formatter.percent(summary.row.share) }}</b>
        </div>
        <div>
          <span>{{ t('modules.combatJournal.analysis.hitEvents') }}</span
          ><b>{{ formatter.amount(summary.hits) }}</b>
        </div>
      </div>
      <p v-if="summary?.row.value === null" class="analysis-note">
        {{ t('modules.combatJournal.partial') }}
      </p>
      <div
        v-if="
          view.tab !== JournalAnalysisTab.Overview &&
          view.tab !== JournalAnalysisTab.Effects &&
          view.tab !== JournalAnalysisTab.Deaths &&
          !(view.tab === JournalAnalysisTab.Rotation && view.raidCasts)
        "
        class="analysis-filters"
      >
        <UiSelect
          v-if="view.tab !== JournalAnalysisTab.Compare"
          :label="t('modules.combatJournal.analysis.target')"
          :model-value="view.target"
          :options="targetOptions"
          @update:model-value="
            view.target = $event;
            view.skill = null;
          "
        />
        <input
          v-if="view.tab === JournalAnalysisTab.Skills || view.tab === JournalAnalysisTab.Compare"
          v-model="view.search"
          class="ui-input"
          :aria-label="t('modules.combatJournal.analysis.search')"
          :placeholder="t('modules.combatJournal.analysis.search')"
        />
      </div>
      <div
        v-if="
          view.skill !== null &&
          view.tab !== JournalAnalysisTab.Overview &&
          !(view.tab === JournalAnalysisTab.Rotation && view.raidCasts)
        "
        class="analysis-active-filter"
      >
        <span>{{ selected?.row.label ?? formatter.skill(Number(view.skill)) }}</span
        ><UiButton @click="view.skill = null">{{
          t('modules.combatJournal.analysis.allSkills')
        }}</UiButton>
      </div>
      <JournalOverview
        v-if="view.tab === JournalAnalysisTab.Overview"
        :segment="segment"
        :window="analysisWindow"
        :query="query"
        :formatter="formatter"
        :timeline-available="timelineAvailable"
        @select="selectActor"
        @death="selectDeath"
      />
      <template v-else-if="view.tab === JournalAnalysisTab.Skills">
        <div v-if="timelineAvailable" class="analysis-chart-modes">
          <button type="button" :aria-pressed="!view.bySkill" @click="view.bySkill = false">
            {{ t('modules.combatJournal.analysis.totalChart') }}</button
          ><button type="button" :aria-pressed="view.bySkill" @click="view.bySkill = true">
            {{ t('modules.combatJournal.analysis.abilityChart') }}
          </button>
        </div>
        <JournalTimeline
          v-if="timelineAvailable"
          :key="chartKey"
          :series="view.bySkill ? abilitySeries : chartSeries"
          :stacked="view.bySkill"
          :duration="duration"
          :formatter="formatter"
        />
        <p v-if="timelineAvailable && !full" class="analysis-warning">
          {{ t('modules.combatJournal.analysis.tracePartial') }}
        </p>
        <div class="analysis-table-tools">
          <UiSelect
            :label="t('modules.combatJournal.analysis.sort')"
            :model-value="view.sort"
            :options="
              Object.values(JournalAnalysisSort).map((value) => ({
                value,
                label: t('modules.combatJournal.analysis.sorts.' + value),
              }))
            "
            @update:model-value="view.sort = $event"
          />
        </div>
        <JournalSkillsTable
          :entries="visible"
          :skill="view.skill"
          :healing="healing"
          :timeline-available="timelineAvailable"
          :cast-count="castCount"
          :formatter="formatter"
          @skill="view.skill = $event"
          @tab="view.tab = $event"
        />
      </template>
      <template v-else-if="view.tab === JournalAnalysisTab.Targets">
        <button
          v-for="row in targetRows"
          :key="row.id"
          type="button"
          class="analysis-target"
          @click="
            view.target = row.id;
            view.tab = JournalAnalysisTab.Skills;
          "
        >
          <span>{{ row.label }}</span
          ><b>{{ formatter.value(row, false, view.metric) }}</b
          ><small>{{ formatter.percent(row.share) }}</small>
        </button></template
      >
      <template v-else-if="view.tab === JournalAnalysisTab.Timeline">
        <p v-if="!full" class="analysis-warning">
          {{ t('modules.combatJournal.analysis.tracePartial') }}
        </p>
        <JournalTimeline
          :key="chartKey"
          :series="chartSeries"
          :duration="duration"
          :formatter="formatter"
          @range="(start, end) => (view.interval = [start, end])"
        />
        <JournalIntervalPanel
          :actions="timelineEvents"
          :formatter="formatter"
          :start="view.interval[0]"
          :end="view.interval[1]"
        />
        <JournalEventsPanel :actions="timelineEvents" :segment="segment" :formatter="formatter" />
      </template>
      <template v-else-if="view.tab === JournalAnalysisTab.Rotation">
        <JournalCastsPanel
          :key="segment.id + ':' + String(view.actor) + ':' + String(view.raidCasts)"
          :actions="view.raidCasts ? groupActions : rotationActions"
          :group-actors="view.raidCasts"
          :actors="castActors"
          :actor-for="castActor"
          :segment="segment"
          :formatter="formatter"
          @actor="
            chooseActor($event);
            view.raidCasts = false;
          "
          @skill="
            view.skill = $event === null ? null : String($event);
            view.tab = JournalAnalysisTab.Events;
          "
        />
      </template>
      <JournalDeathsPanel
        v-else-if="view.tab === JournalAnalysisTab.Deaths"
        :segment="segment"
        :actor="view.actor"
        :formatter="formatter"
      />
      <JournalEventsPanel
        v-else-if="view.tab === JournalAnalysisTab.Events"
        :actions="eventActions"
        :segment="segment"
        :formatter="formatter"
      />
      <JournalEffectsPanel
        v-else-if="view.tab === JournalAnalysisTab.Effects"
        :segment="segment"
        :actor="view.actor"
        :formatter="formatter"
      />
      <JournalComparePanel
        v-else
        :segment="segment"
        :segments="segments"
        :entries="entries"
        :window="analysisWindow"
        :query="query"
        :formatter="formatter"
        :search="view.search"
        :actor-label="actorRow?.label ?? null"
        :reference="view.reference"
        :reference-actor="view.referenceActor"
        @reference="
          (fight, actor) => {
            view.reference = fight;
            view.referenceActor = actor;
          }
        "
        @reference-actor="view.referenceActor = $event"
      />
      <p v-if="exportError" role="alert">{{ exportError }}</p>
    </div>
  </div>
</template>
<style scoped>
.analysis-history {
  display: flex;
  gap: 6px;
  padding-left: 20px;
  flex-shrink: 0;
}
.analysis-toolbar {
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--prism-border);
  flex-shrink: 0;
}
.analysis-toolbar :deep(.fight-picker) {
  flex: 1;
  min-width: 0;
  border-bottom: 0;
}
.analysis-scope {
  padding-top: 12px !important;
}
.analysis-scope :deep(.ui-switch) {
  justify-content: space-between;
  flex-direction: row-reverse;
  border: 1px solid var(--prism-border);
  border-radius: 4px;
  padding: 6px 10px;
}
.analysis-chart-modes {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
}
.analysis-chart-modes button {
  border: 1px solid var(--prism-border);
  border-radius: 4px;
  padding: 5px 10px;
  background: transparent;
  color: var(--prism-muted);
  font: inherit;
  font-size: 11px;
  cursor: pointer;
}
.analysis-chart-modes button[aria-pressed='true'] {
  color: var(--prism-text);
  background: var(--prism-selected);
  border-color: var(--prism-accent);
}
.journal-analysis {
  display: flex;
  flex-direction: column;
  min-height: 0;
  width: 100%;
  color: var(--prism-text);
  font:
    13px 'Inter Variable',
    system-ui,
    sans-serif;
}
.analysis-context {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 16px 20px;
}
.analysis-context strong {
  font-size: 18px;
}
.analysis-context small {
  display: block;
  color: var(--prism-muted);
  margin-top: 6px;
}
.analysis-summary {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 1px;
  background: var(--prism-border);
  margin: 0 20px 16px;
  border: 1px solid var(--prism-border);
}
.analysis-summary > div {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 12px;
  background: var(--prism-panel-background);
}
.analysis-summary span {
  color: var(--prism-muted);
  font-size: 12px;
}
.analysis-summary b {
  font-size: 19px;
  font-variant-numeric: tabular-nums;
}
.analysis-filters {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  padding: 0 20px 14px;
}
.analysis-tabs {
  display: flex;
  gap: 20px;
  padding: 0 20px;
  border-bottom: 1px solid var(--prism-border);
  overflow-x: auto;
  flex-shrink: 0;
}
.analysis-tabs button {
  padding: 10px 0;
  border: 0;
  border-bottom: 2px solid transparent;
  border-radius: 0;
  background: none;
  color: var(--prism-muted);
  white-space: nowrap;
  cursor: pointer;
}
.analysis-tabs button:disabled {
  opacity: 0.4;
  cursor: default;
}
.analysis-tabs button[aria-pressed='true'] {
  border-bottom-color: var(--prism-accent);
  color: var(--prism-text);
}
.analysis-scroll {
  overflow: auto;
  min-height: 0;
  padding: 16px 20px;
}
.analysis-scroll-rotation {
  display: flex;
  flex: 1;
  flex-direction: column;
  overflow: hidden;
}
.analysis-scroll > .analysis-context {
  padding: 0 0 14px;
}
.analysis-scroll > .analysis-filters {
  padding: 0 0 14px;
}
.analysis-scroll > .analysis-summary {
  margin: 0 0 16px;
}
.analysis-scroll-rotation > :not(.journal-casts-panel) {
  flex-shrink: 0;
}
.analysis-table-tools {
  margin-top: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  margin-bottom: 12px;
  color: var(--prism-muted);
  font-size: 11px;
}
.analysis-table-tools :deep(.ui-select) {
  flex: 0 0 180px;
  width: 180px;
  min-width: 0;
}
.analysis-note {
  font-size: 12px;
  line-height: 1.5;
  color: var(--prism-muted);
  margin: 0 0 12px;
}
.analysis-warning {
  color: var(--prism-accent);
  font-size: 12px;
  line-height: 1.5;
}
.analysis-target {
  display: flex;
  width: 100%;
  justify-content: space-between;
  gap: 15px;
  text-align: left;
  padding: 12px;
  background: var(--prism-field);
  color: inherit;
  border: 0;
  border-bottom: 1px solid var(--prism-border);
  cursor: pointer;
}
.analysis-target span {
  flex: 1;
}
.analysis-target small {
  color: var(--prism-muted);
}
@media (max-width: 650px) {
  .analysis-summary {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .analysis-filters {
    grid-template-columns: 1fr;
  }
}
.analysis-active-filter {
  display: flex;
  align-items: center;
  gap: 12px;
}
.analysis-active-filter {
  padding: 8px 20px;
  background: var(--prism-selected);
  font-size: 12px;
}
.analysis-scroll > .journal-events-panel {
  margin-top: 20px;
}
</style>
