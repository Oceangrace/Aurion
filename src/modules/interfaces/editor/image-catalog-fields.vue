<script setup lang="ts">
import { ImageSource } from '../model/image-source';
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { WidgetNode } from '../model/widget-node';
import type { EditorSession } from './editor-session';
import type { CatalogClient } from '../triggers/catalog-client';
import type { CatalogSelection } from '../model/catalog-selection';
import type { CatalogItem } from '../model/catalog-item';
import type { CatalogForm } from '../model/catalog-form';
import RuleCatalogFields from './rule-catalog-fields.vue';
import UiButton from '../../../renderer/components/ui/ui-button.vue';
const props = defineProps<{ widget: WidgetNode; session: EditorSession; catalog: CatalogClient }>();

const { t } = useI18n();

const open = ref(false);

const effect = ref(true);

const automatic = async (): Promise<void> => {
  const widget = props.widget;

  const reference = widget.trigger?.selection ?? Object.values(widget.ruleSelections).flat()[0];

  try {
    const record =
      reference === undefined
        ? null
        : await props.catalog.item(reference.scope, reference.contextKey, reference.entryKey);

    if (props.session.selected()?.id === widget.id) {
      props.session.updateSelected((node) => ({
        ...node,
        imageSelection: null,
        appearance: { ...node.appearance, customImage: null, iconPath: record?.item?.icon ?? null },
        style: {
          ...node.style,
          imageSource: ImageSource.Automatic,
          imageSkillId: null,
          imageEffectId: null,
        },
      }));
    }
  } catch (error) {
    props.session.report(error);
  }
};

const choose = (selection: CatalogSelection, item: CatalogItem, form: CatalogForm): void => {
  props.session.updateSelected((node) => ({
    ...node,
    imageSelection: selection,
    appearance: { ...node.appearance, iconPath: item.icon, customImage: null },
    style: {
      ...node.style,
      imageSource: effect.value ? ImageSource.Effect : ImageSource.Skill,
      imageSkillId: effect.value ? null : (form.matchIds[0] ?? null),
      imageEffectId: effect.value ? (form.matchIds[0] ?? null) : null,
    },
  }));
  open.value = false;
};
</script>
<template>
  <div class="choice-row image-source-actions">
    <UiButton :active="widget.style.imageSource === ImageSource.Automatic" @click="automatic">{{
      t('modules.interfaces.advanced.automaticImage')
    }}</UiButton>
    <UiButton @click="open = !open">{{ t('modules.interfaces.advanced.imageCatalog') }}</UiButton>
  </div>
  <div v-if="open" class="image-source-picker">
    <div class="choice-row">
      <UiButton :active="effect" @click="effect = true">{{
        t('modules.interfaces.rules.types.effect')
      }}</UiButton
      ><UiButton :active="!effect" @click="effect = false">{{
        t('modules.interfaces.skillUsed')
      }}</UiButton>
    </div>
    <RuleCatalogFields
      :catalog="catalog"
      :effect="effect"
      :selected="widget.imageSelection === null ? [] : [widget.imageSelection]"
      @choose="choose"
    />
  </div>
</template>
