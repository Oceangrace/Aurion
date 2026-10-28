<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { WidgetNode } from '../../model/widget-node';
import { WidgetPreset } from './widget-preset';
import { WidgetKind } from '../../model/widget-kind';
import { WidgetGeometry } from '../../scene/widget-geometry';
import { WidgetPresetPreview } from './widget-preset-preview';
import IconWidget from '../../scene/icon-widget.vue';
import BarWidget from '../../scene/bar-widget.vue';
import TextWidget from '../../scene/text-widget.vue';
import WidgetKindIcon from '../widget-kind-icon.vue';

const props = defineProps<{ widget: WidgetNode | null; preset: WidgetPreset; kind: WidgetKind }>();

const { t } = useI18n();

const samples = new WidgetPresetPreview();

const geometry = new WidgetGeometry();

const node = computed(() =>
  props.widget === null ? null : samples.widget(props.widget, props.preset),
);

const visual = computed(() =>
  node.value === null ? null : samples.visual(node.value, props.preset),
);

const style = computed(() =>
  node.value === null ? {} : geometry.style(node.value, { ...node.value.layout, x: 0, y: 0 }),
);
</script>
<template>
  <div
    class="element-preset-preview"
    role="img"
    :aria-label="t('modules.interfaces.presets.preview')"
  >
    <small>{{ t('modules.interfaces.presets.preview') }}</small>
    <div class="element-preset-stage">
      <div
        v-if="preset !== WidgetPreset.Empty && kind === WidgetKind.DynamicGroup"
        class="element-preset-group-preview"
      >
        <WidgetKindIcon v-for="index in 3" :key="index" :kind="WidgetKind.Icon" />
      </div>
      <div
        v-else-if="node !== null && kind !== WidgetKind.Group && kind !== WidgetKind.DynamicGroup"
        class="element-preset-widget"
        :style="style"
      >
        <IconWidget v-if="kind === WidgetKind.Icon" :widget="node" :runtime="visual" />
        <BarWidget v-else-if="kind === WidgetKind.Bar" :widget="node" :runtime="visual" />
        <TextWidget v-else :widget="node" />
      </div>
      <WidgetKindIcon v-else :kind="kind" />
    </div>
  </div>
</template>
