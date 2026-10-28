<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { CombatJournalModule } from '../combat-journal-module';
import type { JournalWindow } from '../model/journal-window';
import type { JournalRow } from '../model/journal-row';
import type { JournalSegment } from '../model/journal-segment';
import type { UiSelectOption } from '../../../renderer/components/ui/ui-select-option';
import { JournalFont } from '../model/journal-font';
import { JournalPeriod } from '../model/journal-period';
import { JournalGestureMode } from '../model/journal-gesture-mode';
import { JournalGesture } from './journal-gesture';
import JournalStatistic from './journal-statistic.vue';
import JournalHeaderIcon from './journal-header-icon.vue';
import JournalAnalysisPanel from './journal-analysis-panel.vue';
import JournalSkillIcon from './journal-skill-icon.vue';
import { JournalMetric } from '../model/journal-metric';
import { JournalDetail } from '../model/journal-detail';
import { JournalSelection } from '../model/journal-selection';
import { CombatCounter } from '../model/combat-counter';
import { JournalQuery } from '../runtime/journal-query';
import FloatingSurface from '../../../renderer/shell/floating-surface.vue';
import UiIconButton from '../../../renderer/components/ui/ui-icon-button.vue';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import type { JournalAppearance } from '../model/journal-appearance';
import { JournalHeaderButton } from '../model/journal-header-button';
const props = defineProps<{
  journal: CombatJournalModule;
  window: JournalWindow;
  appearance: JournalAppearance;
  visible: boolean;
}>();

const { t } = useI18n();

const settingsWindow = props.journal.getSettingsWindow();

const preferences = props.journal.getPreferences();

const settings = computed(() => settingsWindow.value === props.window.id);

const footerSettings = computed(
  () =>
    props.appearance.showHeader === false ||
    props.appearance.headerButtons.includes(JournalHeaderButton.Settings) === false,
);

const toggleSettings = (): void => {
  if (settings.value === true) {
    props.journal.closeSettings();

    return;
  }

  props.journal.openSettings(props.window.id);
};

const previewEnabled = ref(false);

const selected = ref<string>(JournalSelection.Latest);

const detail = ref(JournalDetail.Actors);

const actor = ref<number | null>(null);

const revision = props.journal.getRevision();

const error = props.journal.getError();

const frame = props.journal.getFrame();

const formatter = props.journal.formatter;

const previewQuery = new JournalQuery(
  (id) => props.journal.preview.skill(id, (key) => t('modules.combatJournal.preview.' + key)),
  (id, current) => formatter.unit(id, current),
);

const latest = computed<UiSelectOption<string>>(() => ({
  value: JournalSelection.Latest,
  label: t('modules.combatJournal.periods.' + props.window.period),
}));

const liveSegment = shallowRef<JournalSegment | null>(null);

const previewElapsed = ref(0);

const sample = computed(() =>
  props.journal.preview.create(
    (key) => t('modules.combatJournal.preview.' + key),
    previewElapsed.value,
  ),
);

let previewTimer: ReturnType<typeof setInterval> | null = null;

watch(previewEnabled, (enabled) => {
  if (previewTimer !== null) {
    clearInterval(previewTimer);
    previewTimer = null;
  }

  if (enabled === false) {
    return;
  }

  const started = performance.now();

  previewElapsed.value = 0;
  previewTimer = setInterval(() => {
    previewElapsed.value = performance.now() - started;
  }, 250);
});

const segment = computed(() => (previewEnabled.value ? sample.value : liveSegment.value));

const segments = shallowRef<readonly JournalSegment[]>([]);

watch(
  [revision, selected, () => props.window.period],
  () => {
    liveSegment.value = props.journal.history.read(
      props.window.period,
      selected.value === latest.value.value ? null : selected.value,
    );
    segments.value = props.journal.history.list();
  },
  { immediate: true },
);

const query = computed(() => (previewEnabled.value ? previewQuery : props.journal.query));

const allRows = computed(() =>
  query.value.rows(segment.value, { ...props.window, limit: Number.MAX_SAFE_INTEGER }),
);

const rows = computed(() => allRows.value.slice(0, props.window.limit));

const hovered = ref<number | null>(null);

const tooltip = computed(() =>
  hovered.value === null
    ? []
    : query.value
        .rows(segment.value, props.window, JournalDetail.Skills, hovered.value)
        .slice(0, 6),
);

const total = computed(() => {
  if (props.window.metric === JournalMetric.Critical) {
    const critical = allRows.value.reduce(
      (sum, row) => sum + (row.statistics.counters[CombatCounter.CriticalHits] ?? 0),
      0,
    );

    const hits = allRows.value.reduce(
      (sum, row) => sum + (row.statistics.counters[CombatCounter.DirectHits] ?? 0),
      0,
    );

    return hits === 0 ? 0 : (critical * 100) / hits;
  }

  return allRows.value.reduce((sum, row) => sum + row.knownValue, 0);
});

const partial = computed(() => allRows.value.some((row) => row.value === null));

const selection = computed(() =>
  selected.value === latest.value.value ? props.window.period : selected.value,
);

const choices = computed(() => [
  ...Object.values(JournalPeriod).map((value) => ({
    value,
    label: t('modules.combatJournal.periods.' + value),
  })),
  ...segments.value.map((entry) => ({
    value: entry.id,
    label:
      (entry.period === JournalPeriod.Run
        ? t('modules.combatJournal.periods.run') + ' · ' + (formatter.zone(entry) ?? '—')
        : formatter.npc(entry.npcId)) +
      ' · ' +
      formatter.time(entry.elapsedMs) +
      ' · ' +
      new Date(entry.startedAtUs / 1000).toLocaleTimeString(),
  })),
]);

const metrics = computed(() =>
  Object.values(JournalMetric).map((value) => ({
    value,
    label: t('modules.combatJournal.metrics.' + value),
  })),
);

const select = (value: string): void => {
  const period = new Map<string, JournalPeriod>(
    Object.values(JournalPeriod).map((item) => [item, item]),
  ).get(value);

  selected.value = period === undefined ? value : JournalSelection.Latest;

  if (period !== undefined) {
    update({ period });
  }
};

const fonts: Readonly<Record<JournalFont, string>> = {
  [JournalFont.Lora]: 'Lora Variable, serif',
  [JournalFont.Sans]: 'Arial, sans-serif',
  [JournalFont.Mono]: 'monospace',
  [JournalFont.Inter]: 'Inter Variable, sans-serif',
  [JournalFont.Roboto]: 'Roboto Variable, sans-serif',
  [JournalFont.OpenSans]: 'Open Sans Variable, sans-serif',
  [JournalFont.NotoSans]: 'Noto Sans Variable, sans-serif',
  [JournalFont.SourceSans]: 'Source Sans 3 Variable, sans-serif',
  [JournalFont.UbuntuSans]: 'Ubuntu Sans Variable, sans-serif',
  [JournalFont.Montserrat]: 'Montserrat Variable, sans-serif',
  [JournalFont.RobotoCondensed]: 'Roboto Condensed Variable, sans-serif',
  [JournalFont.Oswald]: 'Oswald Variable, sans-serif',
  [JournalFont.IbmPlexSans]: 'IBM Plex Sans Variable, sans-serif',
  [JournalFont.RobotoMono]: 'Roboto Mono Variable, monospace',
};

const placement = shallowRef<JournalWindow | null>(null);

const shown = computed(() => placement.value ?? props.window);

const color = (value: string, token: string): string =>
  props.appearance.customColors ? value : 'var(' + token + ')';

const style = computed(() => {
  const appearance = props.appearance;

  return {
    left: `clamp(0px, ${String(shown.value.x)}px, calc(100vw - min(${String(shown.value.width)}px, 100vw)))`,
    top: `clamp(0px, ${String(shown.value.y)}px, calc(100vh - min(${String(shown.value.height)}px, 100vh)))`,
    width: `min(${String(shown.value.width)}px, 100vw)`,
    height: `min(${String(shown.value.height)}px, 100vh)`,
    fontSize: String(appearance.fontSize) + 'px',
    fontFamily: fonts[appearance.font],
    color: color(appearance.textColor, '--prism-text'),
    background: `color-mix(in srgb, ${color(appearance.backgroundColor, '--prism-panel-background')} ${String(appearance.opacity * 100)}%, transparent)`,
    border: `${String(appearance.borderWidth)}px ${appearance.borderStyle} ${color(appearance.borderColor, '--prism-panel-border')}`,
    '--journal-header-height': String(appearance.headerHeight) + 'px',
    '--journal-header-font': String(appearance.headerFontSize) + 'px',
    '--journal-header-color': color(appearance.headerTextColor, '--prism-text'),
    '--journal-header-background': `color-mix(in srgb, ${color(appearance.headerColor, '--prism-card')} ${String(appearance.headerOpacity)}%, transparent)`,
    '--journal-row-height': String(appearance.rowHeight) + 'px',
    '--journal-row-gap': String(Math.min(appearance.rowGap, appearance.rowHeight - 4)) + 'px',
    '--journal-row-padding': String(appearance.rowPadding) + 'px',
    '--journal-row-radius': String(appearance.rowRadius) + 'px',
    '--journal-row-background': `color-mix(in srgb, ${color(appearance.rowBackgroundColor, '--prism-field')} ${String(appearance.rowBackgroundOpacity)}%, transparent)`,
    '--journal-animation': String(appearance.animationMs) + 'ms',
    '--journal-padding': String(appearance.padding) + 'px',
    '--journal-value-color': color(appearance.valueColor, '--prism-text'),
    '--journal-total-color': color(appearance.totalColor, '--prism-muted'),
    '--journal-bar-opacity': String(appearance.barOpacity / 100),
    '--journal-button-size': String(appearance.headerButtonSize) + 'px',
  };
});

const gesture = new JournalGesture(
  () => shown.value,
  (value) => {
    placement.value = value;
  },
  (value) => {
    props.journal.update(value);
  },
);

const cancelGesture = (): void => {
  gesture.cancel();
};

watch(
  () => props.visible,
  (visible) => {
    if (visible === false) {
      cancelGesture();
      hovered.value = null;
    }
  },
);

const finishGesture = (): void => {
  gesture.finish(shown.value);
};

onMounted(() => {
  window.addEventListener('blur', cancelGesture);
});

onUnmounted(() => {
  if (previewTimer !== null) {
    clearInterval(previewTimer);
  }

  gesture.cancel();
  window.removeEventListener('blur', cancelGesture);
});

const rateMetric = computed(
  () => props.window.metric === JournalMetric.Dps || props.window.metric === JournalMetric.Hps,
);

const amount = (row: JournalRow): number =>
  row.statistics.counters[
    props.window.metric === JournalMetric.Hps ? CombatCounter.Healing : CombatCounter.Damage
  ] ?? 0;

const resetKey = computed(
  () => String(previewEnabled.value) + ':' + (segment.value?.id ?? '') + ':' + props.window.metric,
);

const totalAmount = computed(() => allRows.value.reduce((sum, row) => sum + amount(row), 0));

const settingsX = computed(() =>
  Math.max(0, Math.min(window.innerWidth - 680, shown.value.x + shown.value.width + 12)),
);

const analysisX = computed(() => Math.max(0, Math.min(window.innerWidth - 1120, settingsX.value)));

const analysisY = computed(() => Math.max(0, Math.min(window.innerHeight - 820, props.window.y)));

const analysisLayout = computed(
  () =>
    preferences.value?.analysisPanel ?? {
      x: analysisX.value,
      y: analysisY.value,
      width: 1120,
      height: 820,
    },
);

const update = (patch: Partial<JournalWindow>): void => {
  props.journal.update({ ...props.window, ...patch });
};

const open = (row: JournalRow): void => {
  if (detail.value === JournalDetail.Actors) {
    actor.value = row.actorId;
    detail.value = JournalDetail.Skills;
  }
};

const back = (): void => {
  actor.value = null;
  detail.value = JournalDetail.Actors;
  hovered.value = null;
};

watch([previewEnabled, selected, () => props.window.period, () => props.window.scope], back);
</script>
<template>
  <section
    v-show="visible"
    class="journal-window"
    :class="{ 'journal-gradient': appearance.barGradient }"
    :style="style"
    :data-journal-window="window.id"
    :data-journal-preview="previewEnabled || undefined"
    data-overlay-interactive
    @pointermove="gesture.move($event)"
    @pointerup="finishGesture"
    @pointercancel="cancelGesture"
    @lostpointercapture="cancelGesture"
  >
    <header
      v-if="appearance.showHeader || settings"
      class="journal-header"
      :class="{
        'journal-locked': window.locked,
        'journal-buttons-on-hover': appearance.headerButtonsOnHover,
      }"
      @pointerdown="gesture.begin($event, JournalGestureMode.Move)"
    >
      <template v-for="button in appearance.headerButtons" :key="button">
        <UiSelect
          v-if="button === JournalHeaderButton.Metric"
          class="journal-metric"
          :model-value="window.metric"
          :options="metrics"
          :disabled="visible === false"
          :label="t('modules.combatJournal.metric')"
          :display-label="t('modules.combatJournal.shortMetrics.' + window.metric)"
          compact
          @update:model-value="update({ metric: $event })"
        />
        <UiSelect
          v-else-if="button === JournalHeaderButton.Segment"
          class="journal-segment"
          :model-value="selection"
          :options="choices"
          :disabled="previewEnabled || visible === false"
          :label="t('modules.combatJournal.history')"
          :display-label="
            window.period === JournalPeriod.Current ? formatter.time(segment?.elapsedMs ?? 0) : 'Σ'
          "
          compact
          @update:model-value="select"
        />
        <UiIconButton
          v-else-if="button === JournalHeaderButton.NewWindow"
          class="journal-header-button"
          :label="t('modules.combatJournal.duplicate')"
          @click="journal.duplicate(window.id)"
        >
          <JournalHeaderIcon :button="button" />
        </UiIconButton>
        <UiIconButton
          v-else-if="button === JournalHeaderButton.Preview"
          class="journal-header-button"
          :label="t('modules.combatJournal.preview.toggle')"
          :active="previewEnabled"
          :aria-pressed="previewEnabled"
          @click="previewEnabled = !previewEnabled"
        >
          <JournalHeaderIcon :button="button" />
        </UiIconButton>
        <UiIconButton
          v-else-if="button === JournalHeaderButton.NewRun"
          class="journal-header-button"
          :label="t('modules.combatJournal.newRun')"
          :disabled="previewEnabled"
          @click="journal.history.resetRun()"
        >
          <JournalHeaderIcon :button="button" />
        </UiIconButton>
        <UiIconButton
          v-else-if="button === JournalHeaderButton.Settings"
          class="journal-header-button"
          :label="t('modules.combatJournal.settings')"
          :active="settings"
          :aria-pressed="settings"
          @click="toggleSettings"
        >
          <JournalHeaderIcon :button="button" />
        </UiIconButton>
        <UiIconButton
          v-else-if="button === JournalHeaderButton.Close"
          class="journal-header-button"
          :label="
            t('shell.closePanel', { title: t('modules.combatJournal.metrics.' + window.metric) })
          "
          @click="journal.close(window.id)"
        >
          <JournalHeaderIcon :button="button" />
        </UiIconButton>
        <span v-if="button === JournalHeaderButton.Metric" class="journal-header-spacer" />
      </template>
      <span
        v-if="appearance.headerButtons.includes(JournalHeaderButton.Metric) === false"
        class="journal-header-spacer"
      />
    </header>
    <div class="journal-body">
      <p v-if="previewEnabled === false && error !== null" class="journal-note" role="alert">
        {{ error }}
      </p>
      <p v-if="previewEnabled === false && frame?.error" class="journal-note">{{ frame.error }}</p>
      <p v-if="segment === null || rows.length === 0" class="journal-note">
        {{ t('modules.combatJournal.empty') }}
      </p>
      <div class="journal-rows" @pointerleave="hovered = null">
        <button
          v-for="(row, index) in rows"
          :key="row.id"
          type="button"
          class="journal-row"
          :data-actor="row.id"
          :style="{
            top: String(index * appearance.rowHeight) + 'px',
            '--row-color': formatter.color(row, appearance),
          }"
          @click="open(row)"
          @pointerenter="hovered = row.actorId"
        >
          <span
            v-if="appearance.bars"
            class="journal-fill"
            :style="{ width: String(row.bar) + '%' }"
          />
          <span class="journal-name"
            ><template v-if="appearance.showRank">{{ index + 1 }}. </template>{{ row.label }}</span
          >
          <span class="journal-value"
            ><JournalStatistic
              :value="rateMetric ? amount(row) : row.knownValue"
              :partial="row.value === null"
              :compact="appearance.compactNumbers"
              :decimals="appearance.decimals"
              :duration="appearance.animationMs"
              :reset-key="resetKey"
            />
            <small v-if="rateMetric">
              (<JournalStatistic
                :value="row.knownValue"
                :partial="row.value === null"
                :compact="appearance.compactNumbers"
                :decimals="appearance.decimals"
                :duration="appearance.animationMs"
                :reset-key="resetKey"
              />
              {{ t('modules.combatJournal.shortMetrics.' + window.metric) }})</small
            >
            <small v-if="window.metric === JournalMetric.Critical">%</small
            ><small v-else-if="appearance.showPercent">
              ({{ row.share.toFixed(appearance.decimals) }}%)</small
            >
          </span>
        </button>
      </div>
    </div>
    <div v-if="appearance.showTotal && rows.length !== 0" class="journal-total">
      <span>
        <JournalStatistic
          :value="rateMetric ? totalAmount : total"
          :partial="partial && window.metric !== JournalMetric.Critical"
          :compact="appearance.compactNumbers"
          :decimals="appearance.decimals"
          :duration="appearance.animationMs"
          :reset-key="resetKey"
        />
        <small v-if="rateMetric">
          (<JournalStatistic
            :value="total"
            :partial="partial"
            :compact="appearance.compactNumbers"
            :decimals="appearance.decimals"
            :duration="appearance.animationMs"
            :reset-key="resetKey"
          />
          {{ t('modules.combatJournal.shortMetrics.' + window.metric) }})</small
        ><small v-if="window.metric === JournalMetric.Critical">%</small>
      </span>
    </div>
    <footer class="journal-footer">
      <button
        type="button"
        class="journal-lock"
        :aria-label="t('modules.combatJournal.lock')"
        :aria-pressed="window.locked"
        @click="update({ locked: !window.locked })"
      >
        <svg viewBox="0 0 20 20">
          <path d="M5 9h10v8H5Z" />
          <path :d="window.locked ? 'M7 9V6a3 3 0 0 1 6 0v3' : 'M7 9V6a3 3 0 0 1 6 0'" />
        </svg>
      </button>
      <button
        v-if="footerSettings"
        type="button"
        class="journal-control"
        :aria-label="t('modules.combatJournal.settings')"
        @click="toggleSettings"
      >
        ⚙
      </button>
      <span v-if="previewEnabled" class="journal-preview-label">{{
        t('modules.combatJournal.preview.notice')
      }}</span
      ><span v-else-if="partial" class="journal-partial" :title="t('modules.combatJournal.partial')"
        >≥</span
      >
      <button
        v-if="window.locked === false"
        type="button"
        class="journal-resize-height"
        :aria-label="t('modules.combatJournal.resizeHeight')"
        @pointerdown="gesture.begin($event, JournalGestureMode.Height)"
      >
        —
      </button>
      <button
        v-if="window.locked === false"
        type="button"
        class="journal-resize-size"
        :aria-label="t('modules.combatJournal.resize')"
        @pointerdown="gesture.begin($event, JournalGestureMode.Size)"
      >
        ◢
      </button>
    </footer>
    <aside v-if="tooltip.length !== 0 && actor === null" class="journal-tooltip">
      <strong>{{ t('modules.combatJournal.details.skills') }}</strong>
      <div v-for="row in tooltip" :key="row.id">
        <span class="journal-tooltip-skill" :title="row.label + ' · ' + row.id"
          ><JournalSkillIcon :path="formatter.icon(Number(row.id))" />{{ row.label }}</span
        >
        <span class="journal-detail-value">
          <b v-if="rateMetric"
            >{{ row.value === null ? '≥ ' : ''
            }}{{ formatter.number(amount(row), appearance.compactNumbers) }}</b
          >
          <small v-if="rateMetric"
            >{{ formatter.value(row, appearance.compactNumbers, window.metric) }}
            {{ t('modules.combatJournal.metrics.' + window.metric) }}</small
          >
          <b v-else>{{ formatter.value(row, appearance.compactNumbers, window.metric) }}</b>
        </span>
      </div>
    </aside>
  </section>
  <FloatingSurface
    v-if="actor !== null && segment !== null"
    class="journal-details-panel"
    :title="t('modules.combatJournal.analysis.title')"
    :x="analysisLayout.x"
    :y="analysisLayout.y"
    :width="analysisLayout.width"
    :height="analysisLayout.height"
    :min-width="460"
    :min-height="350"
    @close="back"
    @resized="journal.savePanel('analysisPanel', $event)"
    @moved="journal.savePanel('analysisPanel', { ...analysisLayout, ...$event })"
  >
    <JournalAnalysisPanel
      :segment="segment"
      :segments="segments"
      :actor="actor"
      :window="window"
      :query="query"
      :formatter="formatter"
      :export-file="(text) => journal.exportReport(text)"
    />
  </FloatingSurface>
</template>
<style scoped>
.journal-tooltip-skill {
  display: flex;
  align-items: center;
  gap: 8px;
}
.journal-tooltip {
  font-family: 'Inter Variable', system-ui, sans-serif;
}
.journal-details-panel :deep(.surface-content) {
  flex-direction: column;
  overflow: hidden;
}
.journal-header .journal-control:hover {
  background: var(--prism-selected);
  border-radius: var(--prism-radius-control);
}

.journal-window {
  position: fixed;
  display: flex;
  flex-direction: column;
  pointer-events: auto;
  z-index: 9000;
  max-width: 100vw;
  max-height: 100vh;
  box-sizing: border-box;
  border-radius: var(--prism-radius-card);
  box-shadow: 0 8px 28px #0005;
}
.journal-header {
  --journal-control-height: max(var(--journal-button-size), calc(var(--journal-header-font) + 4px));
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
  min-height: max(var(--journal-header-height), calc(var(--journal-control-height) + 8px));
  flex-shrink: 0;
  padding: 3px 6px;
  box-sizing: border-box;
  font-size: var(--journal-header-font);
  font-family: 'Inter Variable', 'Segoe UI', sans-serif;
  color: var(--journal-header-color);
  background: var(--journal-header-background);
  cursor: move;
  border-radius: var(--prism-radius-card) var(--prism-radius-card) 0 0;
  border-bottom: 1px solid var(--prism-panel-border);
  touch-action: none;
}
.journal-locked {
  cursor: default;
}
.journal-metric {
  flex: 0 0 auto;
  width: auto;
  max-width: 104px;
  min-width: 0;
  height: var(--journal-control-height);
}
.journal-segment {
  width: auto;
  height: var(--journal-control-height);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}
.journal-header-spacer {
  flex: 1;
  min-width: 0;
}
.journal-header .journal-header-button {
  width: var(--journal-button-size);
  height: var(--journal-control-height);
  min-width: 0;
  min-height: 0;
  padding: 0;
  line-height: 1;
  flex-shrink: 0;
}
.journal-header-button :deep(svg) {
  width: calc(var(--journal-button-size) * 0.65);
  height: calc(var(--journal-button-size) * 0.65);
}
.journal-buttons-on-hover .journal-header-button,
.journal-buttons-on-hover .journal-segment {
  opacity: 0;
  transition: opacity 120ms;
}
.journal-window:hover .journal-buttons-on-hover .journal-header-button,
.journal-window:hover .journal-buttons-on-hover .journal-segment,
.journal-buttons-on-hover:focus-within .journal-header-button,
.journal-buttons-on-hover:focus-within .journal-segment {
  opacity: 1;
}
.journal-header :deep(.ui-select-trigger) {
  min-height: 0;
  height: 100%;
  gap: 4px;
  padding: 0 4px;
  font-size: inherit;
  line-height: 1;
  border: 0;
  background: transparent;
  color: inherit;
}
.journal-header :deep(.ui-select-trigger svg) {
  position: static;
  flex: 0 0 12px;
  width: 12px;
  height: 12px;
  padding: 0;
  transform: none;
}
.journal-control,
.journal-footer button {
  border: 0;
  background: transparent;
  color: inherit;
  padding: 0;
  cursor: pointer;
}
.journal-control {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
}
.journal-control[aria-pressed='true'],
.journal-lock[aria-pressed='true'] {
  color: var(--prism-accent);
}
.journal-footer svg {
  width: 13px;
  height: 13px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.2;
}
.journal-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  margin: var(--journal-padding) var(--journal-padding) 0;
}
.journal-rows {
  position: relative;
  height: 100%;
}
.journal-row {
  display: flex;
  align-items: center;
  position: absolute;
  left: 0;
  right: 0;
  width: 100%;
  height: calc(var(--journal-row-height) - var(--journal-row-gap));
  gap: 6px;
  padding: 0 var(--journal-row-padding);
  border: 0;
  border-radius: var(--journal-row-radius);
  background: var(--journal-row-background);
  color: inherit;
  font: inherit;
  overflow: hidden;
  text-align: left;
  cursor: pointer;
  transition: top var(--journal-animation) ease;
}
.journal-fill {
  position: absolute;
  inset: 0 auto 0 0;
  background: var(--row-color);
  opacity: var(--journal-bar-opacity);
  transition: width var(--journal-animation) ease;
  pointer-events: none;
}
.journal-gradient .journal-fill {
  background: linear-gradient(
    90deg,
    var(--row-color),
    color-mix(in srgb, var(--row-color) 45%, transparent)
  );
}
.journal-name {
  flex: 1;
  min-width: 0;
  position: relative;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.journal-value {
  position: relative;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
  color: var(--journal-value-color);
}
.journal-value small,
.journal-total small {
  font-size: inherit;
  opacity: 0.8;
}
.journal-total {
  display: flex;
  justify-content: flex-end;
  flex-shrink: 0;
  margin: 0 var(--journal-padding);
  height: 20px;
  border-top: 1px solid var(--prism-border);
  color: var(--journal-total-color);
  font-variant-numeric: tabular-nums;
}
.journal-footer {
  position: relative;
  height: 16px;
  display: flex;
  flex-shrink: 0;
  color: var(--prism-muted);
}
.journal-footer button {
  display: flex;
  align-items: center;
  justify-content: center;
}
.journal-lock {
  width: 18px;
}
.journal-preview-label {
  position: absolute;
  inset: 0 25px;
  text-align: center;
  color: var(--prism-accent);
  font:
    9px Arial,
    sans-serif;
  pointer-events: none;
  white-space: nowrap;
  overflow: hidden;
}
.journal-partial {
  position: absolute;
  right: 20px;
  pointer-events: none;
}
.journal-resize-height {
  flex: 1;
  cursor: ns-resize !important;
  touch-action: none;
}
.journal-resize-size {
  width: 18px;
  margin-left: auto;
  cursor: nwse-resize !important;
  touch-action: none;
}
.journal-note {
  margin: 0;
  font-size: inherit;
  color: var(--prism-muted);
}
.journal-tooltip {
  position: absolute;
  left: 0;
  top: calc(100% + 8px);
  width: min(320px, 90vw);
  padding: 8px;
  border: 1px solid var(--prism-border);
  background: var(--prism-surface);
  color: var(--prism-text);
  pointer-events: none;
  z-index: 1;
  box-sizing: border-box;
}
.journal-tooltip > div,
.journal-detail-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-top: 4px;
}
.journal-tooltip span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.journal-history-actions {
  flex-shrink: 0;
  flex-wrap: wrap;
  align-items: center;
  display: flex;
  gap: 12px;
  padding: 16px;
  border-top: 1px solid var(--prism-border);
}
.journal-detail-tabs {
  flex-shrink: 0;
  display: flex;
  gap: 16px;
  border-bottom: 1px solid var(--prism-border);
  padding: 0 16px;
}
.journal-detail-tabs button {
  border-radius: 0;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--prism-muted);
  padding: 12px 0;
  cursor: pointer;
}
.journal-detail-tabs button[aria-pressed='true'] {
  color: var(--prism-text);
  border-bottom-color: var(--prism-accent);
}
.journal-detail-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 16px;
}
.journal-detail-row > span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.journal-detail-value {
  flex-shrink: 0;
  display: flex;
  align-items: flex-end;
  flex-direction: column;
  font-variant-numeric: tabular-nums;
}
.journal-detail-value small {
  color: var(--prism-muted);
  font-size: 0.85em;
}
.journal-detail-row > b {
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}
.journal-detail-row {
  width: 100%;
  padding: 8px;
  border: 0;
  color: var(--prism-text);
  background: var(--prism-field);
  cursor: pointer;
  text-align: left;
}
.journal-breakdown {
  margin-top: 16px;
}
.journal-breakdown dl {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 6px;
}
.journal-breakdown dd {
  margin: 0;
  font-variant-numeric: tabular-nums;
}
@media (prefers-reduced-motion: reduce) {
  .journal-row,
  .journal-fill {
    transition: none;
  }
}
</style>
