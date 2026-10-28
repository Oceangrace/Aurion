<script setup lang="ts">
import UiMessage from '../../../renderer/components/ui/ui-message.vue';
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { CatalogClient } from './catalog-client';
import type { CatalogScope } from '../model/catalog-scope';
import { CatalogSource } from '../model/catalog-source';
import type { CatalogItem } from '../model/catalog-item';
import type { CatalogForm } from '../model/catalog-form';
import { CatalogPickerController } from './catalog-picker-controller';
import { CatalogChoice } from './catalog-choice';
import { CatalogOptions } from './catalog-options';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import UiField from '../../../renderer/components/ui/ui-field.vue';
import UiButton from '../../../renderer/components/ui/ui-button.vue';
import './catalog-picker.css';
const props = defineProps<{
  client: CatalogClient;
  scope: CatalogScope;
  contextKey: string | null;
  label: string;
  selectedKey?: string | null;
  choiceKey?: string | null;
  enemies?: boolean;
  category?: string | null;
  source?: CatalogSource;
  grouped?: boolean;
}>();

const emit = defineEmits<{ choose: [item: CatalogItem, form: CatalogForm, version: string] }>();

const { t, locale } = useI18n();

const controller = new CatalogPickerController(props.client);

const choices = new CatalogChoice();

const catalogOptions = new CatalogOptions(t, () => locale.value);

const state = controller.getState();

const query = ref('');

const options = computed(() =>
  catalogOptions.list(state.value.page.items, props.scope, props.grouped === true),
);

const label = computed(() => state.value.selected?.name ?? props.label);

watch(
  () => [props.scope, props.contextKey, props.category, props.source, query.value, locale.value],
  () => {
    controller.schedule({
      scope: props.scope,
      contextKey: props.contextKey,
      text: query.value,
      category: props.category ?? null,
      source: props.source ?? CatalogSource.All,
    });
  },
  { immediate: true },
);

watch(
  () => [props.scope, props.contextKey, props.selectedKey, locale.value],
  () => {
    void controller.restore(props.scope, props.contextKey, props.selectedKey ?? null);
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  controller.dispose();
});

const pick = (key: string): void => {
  const item = state.value.page.items.find((item) => item.key === key);

  if (item === undefined) {
    return;
  }

  const form = choices.all(item);

  if (form !== null) {
    controller.select(item);
    emit('choose', item, form, state.value.page.catalogVersion);
  }
};

const more = (): void => {
  void controller.more();
};
</script>
<template>
  <div class="catalog-picker">
    <UiField :label="props.label">
      <UiSelect
        :model-value="selectedKey ?? ''"
        :display-label="label"
        :options="options"
        :label="props.label"
        :remote-search="true"
        @search="query = $event"
        @update:model-value="pick"
      >
        <template v-if="state.page.total > state.page.items.length" #footer>
          <div class="catalog-picker-footer">
            <span class="hint">
              {{
                t('modules.interfaces.moreResults', {
                  total: state.page.total,
                  shown: state.page.items.length,
                })
              }}
            </span>
            <UiButton :disabled="state.loading" @click="more">
              {{ t('modules.interfaces.loadMore') }}
            </UiButton>
          </div>
        </template>
      </UiSelect>
    </UiField>
    <p v-if="state.restoring" class="hint">{{ t('modules.interfaces.loading') }}</p>
    <p v-if="state.error !== null" role="alert" class="error-note">
      <UiMessage :text="state.error" />
    </p>
    <p
      v-else-if="selectedKey != null && state.selected === null && !state.restoring"
      role="alert"
      class="error-note"
    >
      {{ t('modules.interfaces.unsupported') }}
    </p>
  </div>
</template>
