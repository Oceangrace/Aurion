<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { DataKind } from '../model/rules/data-kind';
import type { FormulaDefinition } from '../model/rules/formula-definition';
import type { TriggerRule } from '../model/rules/trigger-rule';
import { FieldCatalog } from './field-catalog';
import { FieldLabels } from './field-labels';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
const props = withDefaults(
  defineProps<{
    formulas?: readonly FormulaDefinition[];
    rules?: readonly TriggerRule[];
    modelValue?: string;
  }>(),
  { formulas: () => [], rules: () => [], modelValue: '' },
);

const emit = defineEmits<{ choose: [path: string] }>();

const { t, locale } = useI18n();

const labels = new FieldLabels(t);

const catalog = new FieldCatalog();

const options = computed(() => [
  { value: '', label: t('modules.interfaces.insertData') },
  ...catalog
    .entries(props.formulas, props.rules)
    .filter((field) => field.kind !== DataKind.Effects)
    .map((field) => ({
      value: field.path,
      label: labels.label(field.path, props.formulas, props.rules),
      group: labels.group(field),
      parentGroup: labels.parentGroup(field.path),
    }))
    .sort((left, right) => left.label.localeCompare(right.label, locale.value)),
]);

const selectedLabel = computed(
  () =>
    options.value.find((option) => option.value === props.modelValue)?.label ?? props.modelValue,
);
</script>
<template>
  <UiSelect
    :model-value="modelValue"
    :display-label="selectedLabel"
    :options="options"
    :label="t('modules.interfaces.data.title')"
    @update:model-value="
      (value) => {
        if (value !== '') {
          emit('choose', value);
        }
      }
    "
  />
</template>
