<script setup lang="ts">
import { ImageSource } from '../model/image-source';
import UiMessage from '../../../renderer/components/ui/ui-message.vue';
import ClassColorFields from './class-color-fields.vue';
import ImageCatalogFields from './image-catalog-fields.vue';
import type { CatalogClient } from '../triggers/catalog-client';
import AdvancedDisplayFields from './advanced-display-fields.vue';
import GroupBehaviorFields from './group-behavior-fields.vue';
import { computed, ref } from 'vue';
import { WidgetImageLoader } from './widget-image-loader';
import UiButton from '../../../renderer/components/ui/ui-button.vue';
import { useI18n } from 'vue-i18n';
import type { WidgetNode } from '../model/widget-node';
import type { WidgetStyle } from '../model/widget-style';
import type { GroupSettings } from '../model/group-settings';
import type { EditorSession } from './editor-session';
import { InspectorTab } from './inspector-tab';
import { GroupDataSource } from '../model/rules/group-data-source';
import { WidgetKind } from '../model/widget-kind';
import { FontFamily } from '../model/font-family';
import { TextAlignment } from '../model/text-alignment';
import { TextAnchor } from '../model/text-anchor';
import { ImageFit } from '../model/image-fit';
import { WidgetAnimation } from '../model/widget-animation';
import { GroupFlow } from '../model/group-flow';
import UiField from '../../../renderer/components/ui/ui-field.vue';
import UiNumber from '../../../renderer/components/ui/ui-number.vue';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import UiSlider from '../../../renderer/components/ui/ui-slider.vue';
import UiSwitch from '../../../renderer/components/ui/ui-switch.vue';
import UiColor from '../../../renderer/components/ui/ui-color.vue';
import UiInput from '../../../renderer/components/ui/ui-input.vue';
const props = defineProps<{
  widget: WidgetNode;
  session: EditorSession;
  catalog: CatalogClient;
  tab: InspectorTab;
}>();

const { t } = useI18n();

const effectGroup = computed(() => props.widget.binding?.source === GroupDataSource.Effects);

const imageInput = ref<HTMLInputElement | null>(null);

const imageError = ref<string | null>(null);

const imageLoader = new WidgetImageLoader();

const loadImage = async (): Promise<void> => {
  const file = imageInput.value?.files?.[0];

  if (file === undefined) {
    return;
  }

  const id = props.widget.id;

  try {
    const customImage = await imageLoader.load(file);

    if (props.session.selected()?.id === id) {
      props.session.updateSelected((node) => ({
        ...node,
        appearance: { ...node.appearance, customImage },
        style: { ...node.style, imageSource: ImageSource.Custom },
      }));
    }

    imageError.value = null;
  } catch (error) {
    imageError.value = String(error);
  } finally {
    if (imageInput.value !== null) {
      imageInput.value.value = '';
    }
  }
};

const group = computed(() => props.session.tree.isGroup(props.widget));

const fonts = computed(() =>
  Object.values(FontFamily).map((value) => ({
    value,
    label: t('modules.interfaces.visual.fonts.' + value),
  })),
);

const alignments = computed(() =>
  Object.values(TextAlignment).map((value) => ({
    value,
    label: t('modules.interfaces.visual.alignments.' + value),
  })),
);

const verticals = computed(() =>
  Object.values(TextAnchor).map((value) => ({
    value,
    label: t('modules.interfaces.visual.verticals.' + value),
  })),
);

const fits = computed(() =>
  Object.values(ImageFit).map((value) => ({
    value,
    label: t('modules.interfaces.visual.fits.' + value),
  })),
);

const animations = computed(() =>
  Object.values(WidgetAnimation).map((value) => ({
    value,
    label: t('modules.interfaces.visual.animations.' + value),
  })),
);

const flows = computed(() =>
  Object.values(GroupFlow)
    .filter(
      (value) =>
        props.widget.binding?.source !== GroupDataSource.SkillHistory ||
        value === GroupFlow.Timeline,
    )
    .filter((value) => effectGroup.value === false || value !== GroupFlow.Timeline)
    .filter(
      (value) =>
        props.widget.binding?.source !== GroupDataSource.WorldBosses ||
        value !== GroupFlow.Timeline,
    )
    .map((value) => ({
      value,
      label: t('modules.interfaces.visual.flows.' + value),
    })),
);

const flags = ['bold', 'outline', 'shadow', 'wrap'] as const;

const style = (patch: Partial<WidgetStyle>): void => {
  props.session.updateSelected((node) => ({
    ...node,
    style: {
      ...node.style,
      ...patch,
      barDirection: 'barVertical' in patch ? null : node.style.barDirection,
    },
  }));
};

const settings = (patch: Partial<GroupSettings>): void => {
  props.session.updateSelected((node) => ({ ...node, group: { ...node.group, ...patch } }));
};
</script>
<template>
  <ClassColorFields v-if="tab === InspectorTab.Style" :widget="widget" :session="session" />
  <template v-if="group">
    <template v-if="tab === InspectorTab.Display">
      <section class="inspector-card">
        <h3>{{ t('modules.interfaces.visual.group') }}</h3>
        <UiSwitch
          :model-value="widget.group.headerEnabled"
          :label="t('modules.interfaces.visual.headerEnabled')"
          @update:model-value="settings({ headerEnabled: $event })"
        />
        <template v-if="widget.group.headerEnabled">
          <UiField :label="t('modules.interfaces.visual.headerText')"
            ><UiInput
              :model-value="widget.group.headerText"
              :label="t('modules.interfaces.visual.headerText')"
              @update:model-value="settings({ headerText: $event })"
          /></UiField>
        </template>
      </section>
    </template>
    <template v-else-if="tab === InspectorTab.Style">
      <template v-if="widget.group.headerEnabled">
        <section class="inspector-card">
          <h3>{{ t('modules.interfaces.visual.headerText') }}</h3>

          <div class="field-grid">
            <UiField :label="t('modules.interfaces.visual.headerHeight')"
              ><UiNumber
                :model-value="widget.group.headerHeight"
                :label="t('modules.interfaces.visual.headerHeight')"
                :min="20"
                :max="100"
                @update:model-value="settings({ headerHeight: $event })"
            /></UiField>
            <UiField :label="t('modules.interfaces.color')"
              ><UiColor
                :model-value="widget.group.headerColor"
                :label="t('modules.interfaces.color')"
                @update:model-value="settings({ headerColor: $event })"
            /></UiField>
            <UiField :label="t('modules.interfaces.visual.headerBackground')"
              ><UiColor
                :model-value="widget.group.headerBackground"
                :label="t('modules.interfaces.visual.headerBackground')"
                @update:model-value="settings({ headerBackground: $event, headerOpacity: null })"
            /></UiField>
          </div>
          <GroupBehaviorFields
            :widget="widget"
            :session="session"
            :catalog="catalog"
            :tab="InspectorTab.Style"
          />
        </section>
      </template>
    </template>
    <template v-else-if="tab === InspectorTab.Placement">
      <section class="inspector-card">
        <div class="section-intro">
          <h3>{{ t('modules.interfaces.inspectorSections.layout') }}</h3>
          <p class="section-description">{{ t('modules.interfaces.redesign.hints.layout') }}</p>
        </div>
        <UiSwitch
          :model-value="widget.group.locked"
          :label="t('modules.interfaces.visual.locked')"
          @update:model-value="settings({ locked: $event })"
        />
        <UiField
          :label="t('modules.interfaces.visual.groupScale')"
          :hint="t('modules.interfaces.visual.groupScaleHint')"
          ><UiNumber
            :model-value="widget.group.scale"
            :label="t('modules.interfaces.visual.groupScale')"
            :min="25"
            :max="400"
            @update:model-value="settings({ scale: $event })"
        /></UiField>
        <UiField :label="t('modules.interfaces.visual.padding')"
          ><UiNumber
            :model-value="widget.group.padding"
            :label="t('modules.interfaces.visual.padding')"
            :min="0"
            :max="100"
            @update:model-value="settings({ padding: $event })"
        /></UiField>
        <div v-if="widget.kind === WidgetKind.DynamicGroup" class="field-grid">
          <UiField :label="t('modules.interfaces.visual.flow')"
            ><UiSelect
              :model-value="widget.group.flow"
              :options="flows"
              :label="t('modules.interfaces.visual.flow')"
              @update:model-value="settings({ flow: $event })"
          /></UiField>
          <UiField :label="t('modules.interfaces.visual.gap')"
            ><UiNumber
              :model-value="widget.group.gap"
              :label="t('modules.interfaces.visual.gap')"
              :min="0"
              :max="200"
              @update:model-value="settings({ gap: $event })"
          /></UiField>
          <UiField
            v-if="widget.group.flow === GroupFlow.Grid"
            :label="t('modules.interfaces.visual.columns')"
            ><UiNumber
              :model-value="widget.group.columns"
              :label="t('modules.interfaces.visual.columns')"
              :min="1"
              :max="100"
              @update:model-value="settings({ columns: $event })"
          /></UiField>
        </div>
      </section>
    </template>
  </template>
  <template v-if="(!group || effectGroup) && tab === InspectorTab.Style">
    <section class="inspector-card">
      <div class="section-intro">
        <h3>{{ t('modules.interfaces.visual.text') }}</h3>
        <p class="section-description">{{ t('modules.interfaces.redesign.hints.text') }}</p>
      </div>
      <div class="field-grid">
        <UiField
          v-if="session.tree.isGroup(widget) === false || effectGroup"
          :label="t('modules.interfaces.fontSize')"
          ><UiNumber
            :model-value="widget.appearance.fontSize"
            :label="t('modules.interfaces.fontSize')"
            :min="6"
            :max="256"
            @update:model-value="
              session.updateSelected((node) => ({
                ...node,
                appearance: { ...node.appearance, fontSize: $event },
              }))
            "
        /></UiField>
        <UiField :label="t('modules.interfaces.visual.font')"
          ><UiSelect
            :model-value="widget.style.fontFamily"
            :options="fonts"
            :label="t('modules.interfaces.visual.font')"
            @update:model-value="style({ fontFamily: $event })"
        /></UiField>
        <UiField :label="t('modules.interfaces.visual.alignment')"
          ><UiSelect
            :model-value="widget.style.alignment"
            :options="alignments"
            :label="t('modules.interfaces.visual.alignment')"
            @update:model-value="style({ alignment: $event })"
        /></UiField>
        <UiField :label="t('modules.interfaces.visual.vertical')"
          ><UiSelect
            :model-value="widget.style.vertical"
            :options="verticals"
            :label="t('modules.interfaces.visual.vertical')"
            @update:model-value="style({ vertical: $event })"
        /></UiField>
      </div>
      <div class="choice-row">
        <UiSwitch
          v-for="flag in flags"
          :key="flag"
          :model-value="widget.style[flag]"
          :label="t('modules.interfaces.visual.' + flag)"
          @update:model-value="style({ [flag]: $event })"
        />
      </div>
    </section>
  </template>
  <template v-if="widget.kind === WidgetKind.Icon && tab === InspectorTab.Display">
    <section class="inspector-card">
      <div class="section-intro">
        <h3>{{ t('modules.interfaces.types.icon') }}</h3>
        <p class="section-description">{{ t('modules.interfaces.redesign.hints.image') }}</p>
      </div>
      <ImageCatalogFields :widget="widget" :session="session" :catalog="catalog" />
      <div class="choice-row">
        <UiButton @click="imageInput?.click()">{{
          t('modules.interfaces.visual.customImage')
        }}</UiButton>
        <UiButton
          v-if="widget.appearance.customImage !== null"
          @click="
            session.updateSelected((node) => ({
              ...node,
              appearance: { ...node.appearance, customImage: null },
            }))
          "
          >{{ t('modules.interfaces.clearIcon') }}</UiButton
        >
        <UiButton
          v-if="widget.appearance.customImage === null && widget.appearance.iconPath !== null"
          @click="
            session.updateSelected((node) => ({
              ...node,
              appearance: { ...node.appearance, iconPath: null },
            }))
          "
          >{{ t('modules.interfaces.clearIcon') }}</UiButton
        >
        <input
          ref="imageInput"
          hidden
          type="file"
          accept="image/png,image/jpeg,image/webp"
          @change="void loadImage()"
        />
      </div>
      <p v-if="imageError !== null" class="error-note" role="alert">
        <UiMessage :text="imageError" />
      </p>
    </section>
  </template>
  <template v-if="(widget.kind === WidgetKind.Icon || effectGroup) && tab === InspectorTab.Style">
    <section class="inspector-card">
      <h3>{{ t('modules.interfaces.types.icon') }}</h3>
      <UiField :label="t('modules.interfaces.visual.imageFit')"
        ><UiSelect
          :model-value="widget.style.imageFit"
          :options="fits"
          :label="t('modules.interfaces.visual.imageFit')"
          @update:model-value="style({ imageFit: $event })"
      /></UiField>
      <UiSwitch
        :model-value="widget.style.desaturate"
        :label="t('modules.interfaces.visual.desaturate')"
        @update:model-value="style({ desaturate: $event })"
      />
    </section>
  </template>
  <template v-if="widget.kind === WidgetKind.Bar && tab === InspectorTab.Display">
    <section class="inspector-card">
      <h3>{{ t('modules.interfaces.types.bar') }}</h3>
      <UiField :label="t('modules.interfaces.visual.barValue')"
        ><UiSlider
          :model-value="widget.style.barValue"
          :label="t('modules.interfaces.visual.barValue')"
          @update:model-value="style({ barValue: $event })"
      /></UiField>
    </section>
  </template>
  <template v-if="widget.kind === WidgetKind.Bar && tab === InspectorTab.Style">
    <section class="inspector-card">
      <h3>{{ t('modules.interfaces.types.bar') }}</h3>
      <UiField :label="t('modules.interfaces.visual.fillColor')"
        ><UiColor
          :model-value="widget.style.fillColor"
          :label="t('modules.interfaces.visual.fillColor')"
          @update:model-value="style({ fillColor: $event })"
      /></UiField>
      <div class="choice-row">
        <UiSwitch
          :model-value="widget.style.barVertical"
          :label="t('modules.interfaces.visual.barVertical')"
          @update:model-value="style({ barVertical: $event })"
        /><UiSwitch
          :model-value="widget.style.barInverse"
          :label="t('modules.interfaces.visual.barInverse')"
          @update:model-value="style({ barInverse: $event })"
        />
      </div>
    </section>
  </template>
  <template v-if="tab === InspectorTab.Style">
    <section class="inspector-card">
      <div class="section-intro">
        <h3>{{ t('modules.interfaces.visual.decoration') }}</h3>
        <p class="section-description">{{ t('modules.interfaces.redesign.hints.decoration') }}</p>
      </div>
      <div class="field-grid inspector-three-columns">
        <UiField :label="t('modules.interfaces.borderColor')"
          ><UiColor
            :model-value="widget.appearance.borderColor"
            :label="t('modules.interfaces.borderColor')"
            @update:model-value="
              session.updateSelected((node) => ({
                ...node,
                appearance: { ...node.appearance, borderColor: $event },
              }))
            "
        /></UiField>

        <UiField :label="t('modules.interfaces.visual.borderWidth')"
          ><UiNumber
            :model-value="widget.style.borderWidth"
            :label="t('modules.interfaces.visual.borderWidth')"
            :min="0"
            :max="20"
            @update:model-value="style({ borderWidth: $event })"
        /></UiField>
        <UiField :label="t('modules.interfaces.visual.radius')"
          ><UiNumber
            :model-value="widget.style.radius"
            :label="t('modules.interfaces.visual.radius')"
            :min="0"
            :max="100"
            @update:model-value="style({ radius: $event })"
        /></UiField>
      </div>
      <UiField :label="t('modules.interfaces.opacity')"
        ><UiSlider
          :model-value="Math.round(widget.appearance.opacity * 100)"
          :label="t('modules.interfaces.opacity')"
          :min="0"
          :max="100"
          :step="1"
          @update:model-value="
            session.updateSelected((node) => ({
              ...node,
              appearance: { ...node.appearance, opacity: $event / 100 },
            }))
          "
      /></UiField>
    </section>
    <section class="inspector-card">
      <div class="section-intro">
        <h3>{{ t('modules.interfaces.inspectorSections.effects') }}</h3>
        <p class="section-description">{{ t('modules.interfaces.redesign.hints.effects') }}</p>
      </div>
      <div class="field-grid inspector-effects">
        <UiSwitch
          :model-value="widget.style.glow"
          :label="t('modules.interfaces.visual.glow')"
          @update:model-value="style({ glow: $event })"
        />
        <UiField v-if="widget.style.glow" :label="t('modules.interfaces.visual.glowColor')"
          ><UiColor
            :model-value="widget.style.glowColor"
            :label="t('modules.interfaces.visual.glowColor')"
            @update:model-value="style({ glowColor: $event })"
        /></UiField>
        <UiField :label="t('modules.interfaces.visual.animation')"
          ><UiSelect
            :model-value="widget.style.animation"
            :options="animations"
            :label="t('modules.interfaces.visual.animation')"
            @update:model-value="style({ animation: $event })"
        /></UiField>
        <UiField
          v-if="widget.style.animation !== WidgetAnimation.None"
          :label="t('modules.interfaces.visual.periodMs')"
          ><UiNumber
            :model-value="widget.style.periodMs"
            :label="t('modules.interfaces.visual.periodMs')"
            :min="100"
            :max="10000"
            @update:model-value="style({ periodMs: $event })"
        /></UiField>
      </div>
    </section>
  </template>
  <section v-if="group && tab === InspectorTab.Placement" class="inspector-card">
    <h3>{{ t('modules.interfaces.inspectorSections.groupBehavior') }}</h3>
    <GroupBehaviorFields :widget="widget" :session="session" :catalog="catalog" :tab="tab" />
  </section>
  <GroupBehaviorFields
    v-else-if="group && tab === InspectorTab.Display"
    :catalog="catalog"
    :widget="widget"
    :session="session"
    :tab="tab"
  />
  <section v-if="tab === InspectorTab.Style" class="inspector-card advanced-style-section">
    <h3>{{ t('modules.interfaces.advanced.title') }}</h3>
    <div class="inspector-block">
      <AdvancedDisplayFields :widget="widget" :session="session" :tab="tab" />
    </div>
  </section>
  <section v-else-if="tab === InspectorTab.Placement" class="inspector-card">
    <h3>{{ t('modules.interfaces.advanced.title') }}</h3>
    <AdvancedDisplayFields :widget="widget" :session="session" :tab="tab" />
  </section>
  <AdvancedDisplayFields v-else :widget="widget" :session="session" :tab="tab" />
</template>
