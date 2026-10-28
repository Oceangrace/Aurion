<script setup lang="ts">
import { Direction } from '../model/direction';
import FieldPicker from './field-picker.vue';
import TemplateField from './template-field.vue';
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { WidgetNode } from '../model/widget-node';
import type { WidgetStyle } from '../model/widget-style';
import type { EditorSession } from './editor-session';
import { InspectorTab } from './inspector-tab';
import { GroupDataSource } from '../model/rules/group-data-source';
import { WidgetKind } from '../model/widget-kind';
import { TextPosition } from '../model/text-position';
import { GlowMode } from '../model/glow-mode';
import { BorderStyle } from '../model/border-style';
import { ProgressSource } from '../model/progress-source';
import { BarIconAnchor } from '../model/bar-icon-anchor';
import { HealingPredictionTarget } from '../model/healing-prediction-target';
import { BarTexture } from '../model/bar-texture';
import { BarSurfaceStyle } from '../scene/bar-surface-style';
import { AnimationWhen } from '../model/animation-when';
import { AnimationChange } from '../model/animation-change';
import { MotionKind } from '../model/motion-kind';
import UiSlider from '../../../renderer/components/ui/ui-slider.vue';
import UiField from '../../../renderer/components/ui/ui-field.vue';
import UiNumber from '../../../renderer/components/ui/ui-number.vue';
import UiInput from '../../../renderer/components/ui/ui-input.vue';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import UiColor from '../../../renderer/components/ui/ui-color.vue';
import UiSwitch from '../../../renderer/components/ui/ui-switch.vue';
import SoundFields from './sound-fields.vue';
const props = defineProps<{ widget: WidgetNode; session: EditorSession; tab: InspectorTab }>();

const { t } = useI18n();

const surfaceStyle = new BarSurfaceStyle();

const displayKind = computed(() =>
  props.widget.binding?.source === GroupDataSource.Effects ? WidgetKind.Icon : props.widget.kind,
);

const textures = computed(() =>
  Object.values(BarTexture).map((value) => ({
    value,
    label: t('modules.interfaces.healing.textures.' + value),
  })),
);

const style = (patch: Partial<WidgetStyle>): void => {
  const direction = patch.barDirection;

  const next =
    direction === undefined || direction === null
      ? patch
      : { ...patch, barVertical: direction === Direction.Up || direction === Direction.Down };

  props.session.updateSelected((node) => ({ ...node, style: { ...node.style, ...next } }));
};

const options = (values: Readonly<Record<string, string>>): { value: string; label: string }[] =>
  Object.values(values).map((value) => ({
    value,
    label: t('modules.interfaces.advanced.options.' + value),
  }));

const positions = computed(() => options(TextPosition));

const healingTargets = computed(() => [
  { value: '', label: t('modules.interfaces.healing.disabled') },
  ...Object.values(HealingPredictionTarget).map((value) => ({
    value,
    label: t('modules.interfaces.healing.targets.' + value),
  })),
]);

const motions = computed(() => options(MotionKind));

const numbers = [
  { key: 'outlineWidth', min: 0, max: 3, kind: '' },
  { key: 'layer', min: 0, max: 50, kind: '' },
  { key: 'zoom', min: 0, max: 40, kind: WidgetKind.Icon },
  { key: 'warnBelowSeconds', min: 0, max: 3600, kind: WidgetKind.Icon },
  { key: 'rangeMin', min: -1000000000, max: 1000000000, kind: WidgetKind.Bar },
  { key: 'rangeMax', min: -1000000000, max: 1000000000, kind: WidgetKind.Bar },
  { key: 'transitionMs', min: 0, max: 3000, kind: WidgetKind.Bar },
  { key: 'barIconSize', min: 8, max: 256, kind: WidgetKind.Bar },
  { key: 'barIconX', min: -2000, max: 2000, kind: WidgetKind.Bar },
  { key: 'barIconY', min: -2000, max: 2000, kind: WidgetKind.Bar },
  { key: 'barIconPadding', min: 0, max: 100, kind: WidgetKind.Bar },
  { key: 'animationIntensity', min: 0, max: 100, kind: '' },
  { key: 'animationThreshold', min: 0, max: 3600, kind: '' },
  { key: 'animationDuration', min: 100, max: 60000, kind: '' },
  { key: 'motionDuration', min: 50, max: 3000, kind: '' },
] as const;

const numeric = computed(() =>
  numbers.filter((field) => {
    let tab = InspectorTab.Style;

    if (field.key === 'layer') {
      tab = InspectorTab.Placement;
    }

    if (field.key === 'rangeMin' || field.key === 'rangeMax') {
      tab = InspectorTab.Display;
    }

    return tab === props.tab && (field.kind === '' || field.kind === displayKind.value);
  }),
);

const selectors = computed(
  () =>
    [
      { key: 'borderStyle', values: options(BorderStyle), kind: '' },
      { key: 'timerPosition', values: positions.value, kind: WidgetKind.Icon },
      { key: 'stacksPosition', values: positions.value, kind: WidgetKind.Icon },
      { key: 'labelPosition', values: positions.value, kind: WidgetKind.Icon },
      { key: 'glowMode', values: options(GlowMode), kind: WidgetKind.Icon },
      { key: 'source', values: options(ProgressSource), kind: WidgetKind.Bar },
      {
        key: 'barDirection',
        values: Object.values(Direction).map((value) => ({
          value,
          label: t('modules.interfaces.advanced.options.' + value),
        })),
        kind: WidgetKind.Bar,
      },
      { key: 'barIconAnchor', values: options(BarIconAnchor), kind: WidgetKind.Bar },
      { key: 'animationWhen', values: options(AnimationWhen), kind: '' },
      { key: 'animationChange', values: options(AnimationChange), kind: '' },
      { key: 'showMotion', values: motions.value, kind: '' },
      { key: 'hideMotion', values: motions.value, kind: '' },
    ] as const,
);

const selects = computed(() =>
  selectors.value.filter(
    (field) =>
      (field.key === 'source' ? InspectorTab.Display : InspectorTab.Style) === props.tab &&
      (field.kind === '' || field.kind === displayKind.value),
  ),
);

const flags = computed(() => {
  if (props.tab === InspectorTab.Display) {
    return ['showTimer', 'showStacks'] as const;
  }

  return props.tab === InspectorTab.Style ? (['swipe', 'swipeReverse'] as const) : [];
});

const colors = ['outlineColor', 'animationColor', 'sparkColor', 'tickColor'] as const;

const visibleColors = computed(() =>
  colors.filter(
    (key) =>
      props.tab === InspectorTab.Style &&
      (props.widget.kind === WidgetKind.Bar || key === 'outlineColor' || key === 'animationColor'),
  ),
);

const ticks = (value: string): void => {
  const numbers = value
    .split(/[ ,;]+/u)
    .filter((part) => part !== '')
    .map(Number);

  if (numbers.length <= 16 === true && numbers.every(Number.isFinite) === true) {
    style({ ticks: numbers });
  }
};
</script>
<template>
  <section v-if="displayKind === WidgetKind.Icon && flags.length > 0" class="inspector-card">
    <div class="section-intro">
      <h3>{{ t('modules.interfaces.redesign.behavior') }}</h3>
      <p class="section-description">{{ t('modules.interfaces.redesign.behaviorHint') }}</p>
    </div>
    <div class="choice-row icon-behavior">
      <UiSwitch
        v-for="key in flags"
        :key="key"
        :model-value="widget.style[key]"
        :label="t('modules.interfaces.advanced.' + key)"
        @update:model-value="style({ [key]: $event })"
      />
    </div>
  </section>
  <div v-if="selects.length + numeric.length + visibleColors.length > 0" class="field-grid">
    <UiField
      v-for="field in selects"
      :key="field.key"
      :label="t('modules.interfaces.advanced.' + field.key)"
      ><UiSelect
        :model-value="widget.style[field.key] ?? 'right'"
        :options="field.values"
        :label="t('modules.interfaces.advanced.' + field.key)"
        @update:model-value="style({ [field.key]: $event })"
    /></UiField>
    <UiField
      v-for="field in numeric"
      :key="field.key"
      :label="t('modules.interfaces.advanced.' + field.key)"
      ><UiNumber
        :model-value="widget.style[field.key]"
        :label="t('modules.interfaces.advanced.' + field.key)"
        :min="field.min"
        :max="field.max"
        @update:model-value="style({ [field.key]: $event })"
    /></UiField>
    <UiField
      v-for="key in visibleColors"
      :key="key"
      :label="t('modules.interfaces.advanced.' + key)"
      ><UiColor
        :model-value="widget.style[key]"
        :label="t('modules.interfaces.advanced.' + key)"
        @update:model-value="style({ [key]: $event })"
    /></UiField>
  </div>
  <template v-if="widget.kind === WidgetKind.Bar && tab === InspectorTab.Display">
    <UiField :label="t('modules.interfaces.healing.title')">
      <UiSelect
        :model-value="widget.style.healingPrediction ?? ''"
        :options="healingTargets"
        :label="t('modules.interfaces.healing.title')"
        @update:model-value="
          style({ healingPrediction: $event === '' ? null : ($event as HealingPredictionTarget) })
        "
      />
    </UiField>
    <UiField :label="t('modules.interfaces.advanced.path')"
      ><FieldPicker
        :model-value="widget.style.path"
        :formulas="session.document()?.formulas ?? []"
        :rules="widget.rules?.rules ?? []"
        @choose="style({ path: $event })"
    /></UiField>
    <UiField :label="t('modules.interfaces.advanced.rightText')"
      ><TemplateField
        :formulas="session.document()?.formulas ?? []"
        :model-value="widget.style.rightText"
        :label="t('modules.interfaces.advanced.rightText')"
        @update:model-value="style({ rightText: $event })"
    /></UiField>
    <UiField :label="t('modules.interfaces.advanced.ticks')"
      ><UiInput
        :model-value="widget.style.ticks.join(', ')"
        :label="t('modules.interfaces.advanced.ticks')"
        @update:model-value="ticks($event)"
    /></UiField>
  </template>
  <template v-if="widget.kind === WidgetKind.Bar && tab === InspectorTab.Style">
    <UiSwitch
      :model-value="widget.style.barLossColor !== null"
      :label="t('modules.interfaces.healing.showLoss')"
      @update:model-value="style({ barLossColor: $event ? '#d65b62' : null })"
    />
    <div class="field-grid bar-style-fields">
      <UiField
        v-if="widget.style.barLossColor !== null"
        :label="t('modules.interfaces.healing.lossColor')"
      >
        <UiColor
          :model-value="widget.style.barLossColor"
          :label="t('modules.interfaces.healing.lossColor')"
          @update:model-value="style({ barLossColor: $event })"
        />
      </UiField>
      <template v-if="widget.style.healingPrediction !== null">
        <UiField :label="t('modules.interfaces.healing.healingColor')">
          <UiColor
            :model-value="widget.style.healingColor"
            :label="t('modules.interfaces.healing.healingColor')"
            @update:model-value="style({ healingColor: $event })"
          />
        </UiField>
        <UiField :label="t('modules.interfaces.healing.overhealColor')">
          <UiColor
            :model-value="widget.style.overhealColor"
            :label="t('modules.interfaces.healing.overhealColor')"
            @update:model-value="style({ overhealColor: $event })"
          />
        </UiField>
        <UiField :label="t('modules.interfaces.healing.texture')">
          <UiSelect
            :model-value="widget.style.overhealTexture"
            :options="textures"
            :label="t('modules.interfaces.healing.texture')"
            @update:model-value="style({ overhealTexture: $event as BarTexture })"
          />
        </UiField>
        <UiField
          v-if="widget.style.overhealTexture !== BarTexture.Solid"
          :label="t('modules.interfaces.healing.textureSize')"
        >
          <UiNumber
            :model-value="widget.style.overhealTextureSize"
            :min="2"
            :max="32"
            :label="t('modules.interfaces.healing.textureSize')"
            @update:model-value="style({ overhealTextureSize: $event })"
          />
        </UiField>
        <span
          class="healing-texture-preview"
          :title="t('modules.interfaces.healing.texture')"
          :style="
            surfaceStyle.texture(
              widget.style.overhealTexture,
              widget.style.overhealTextureSize,
              widget.style.overhealColor,
            )
          "
        />
      </template>
    </div>
    <UiField :label="t('modules.interfaces.advanced.backgroundOpacity')"
      ><UiSlider
        :model-value="
          widget.style.backgroundOpacity ??
          (widget.appearance.background.length === 9
            ? Math.round(parseInt(widget.appearance.background.slice(7), 16) / 2.55)
            : 100)
        "
        :label="t('modules.interfaces.advanced.backgroundOpacity')"
        @update:model-value="
          session.updateSelected((node) => ({
            ...node,
            style: { ...node.style, backgroundOpacity: $event },
            appearance: {
              ...node.appearance,
              background:
                node.appearance.background.slice(0, 7) +
                Math.round($event * 2.55)
                  .toString(16)
                  .padStart(2, '0'),
            },
          }))
        "
    /></UiField>
    <div class="choice-row">
      <UiSwitch
        :model-value="widget.style.gradient !== null"
        :label="t('modules.interfaces.advanced.gradient')"
        @update:model-value="style({ gradient: $event ? widget.style.fillColor : null })"
      />
      <UiColor
        v-if="widget.style.gradient !== null"
        :model-value="widget.style.gradient"
        :label="t('modules.interfaces.advanced.gradient')"
        @update:model-value="style({ gradient: $event })"
      />
      <UiSwitch
        :model-value="widget.style.spark"
        :label="t('modules.interfaces.advanced.spark')"
        @update:model-value="style({ spark: $event })"
      />
      <UiSwitch
        :model-value="widget.style.barIconEnabled"
        :label="t('modules.interfaces.advanced.barIconEnabled')"
        @update:model-value="style({ barIconEnabled: $event })"
      />
    </div>
  </template>
  <UiField v-if="tab === InspectorTab.Style" :label="t('modules.interfaces.advanced.animationPath')"
    ><FieldPicker
      :model-value="widget.style.animationPath"
      :formulas="session.document()?.formulas ?? []"
      :rules="widget.rules?.rules ?? []"
      @choose="style({ animationPath: $event })"
  /></UiField>
  <SoundFields v-if="tab === InspectorTab.Display" :widget="widget" :session="session" />
</template>
<style scoped>
.healing-texture-preview {
  display: block;
  grid-column: 1 / -1;
  width: 100%;
  height: 24px;
  border: 1px solid var(--border-color, #344240);
}
</style>
