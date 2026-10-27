<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { CatalogClient } from '../../triggers/catalog-client';
import type { CatalogItem } from '../../model/catalog-item';
import type { CatalogForm } from '../../model/catalog-form';
import type { WidgetPresetSelection } from './widget-preset-selection';
import type { EnemyWidgetPresetOptions } from './enemy-widget-preset-options';
import { WidgetPreset } from './widget-preset';
import { CatalogScope } from '../../model/catalog-scope';
import CatalogPicker from '../../triggers/catalog-picker.vue';
import UiField from '../../../../renderer/components/ui/ui-field.vue';
import UiNumber from '../../../../renderer/components/ui/ui-number.vue';
import UiSwitch from '../../../../renderer/components/ui/ui-switch.vue';

const props = defineProps<{
  modelValue: EnemyWidgetPresetOptions;
  preset: WidgetPreset;
  catalog: CatalogClient;
  disabled: boolean;
}>();

const emit = defineEmits<{ 'update:modelValue': [value: EnemyWidgetPresetOptions] }>();

const { t } = useI18n();

const update = (patch: Partial<EnemyWidgetPresetOptions>): void => {
  emit('update:modelValue', { ...props.modelValue, ...patch });
};

const actor = (item: CatalogItem): void => {
  update({ actorKey: item.key, skill: null, expectedSkill: null });
};

const selection = (
  item: CatalogItem,
  form: CatalogForm,
  version: string,
): WidgetPresetSelection => ({
  item,
  selection: {
    catalogVersion: version,
    scope: CatalogScope.EnemySkills,
    contextKey: props.modelValue.actorKey,
    entryKey: item.key,
    choiceKey: form.key,
  },
});

const skill = (item: CatalogItem, form: CatalogForm, version: string): void => {
  const chosen = selection(item, form, version);

  const expected = props.modelValue.expectedSkill;

  const same = expected?.selection.entryKey === props.modelValue.skill?.selection.entryKey;

  update({ skill: chosen, expectedSkill: expected === null || same ? chosen : expected });
};

const countdown = (seconds: number): void => {
  update({
    countdownSeconds: seconds,
    warningSeconds: Math.min(seconds, props.modelValue.warningSeconds),
  });
};
</script>
<template>
  <div v-if="!disabled" class="element-enemy-preset-fields">
    <CatalogPicker
      :client="catalog"
      :scope="CatalogScope.Enemies"
      :context-key="null"
      :selected-key="modelValue.actorKey"
      :label="t('modules.interfaces.findEnemy')"
      :enemies="true"
      @choose="actor"
    />
    <template v-if="modelValue.actorKey !== null">
      <CatalogPicker
        :key="modelValue.actorKey + ':source'"
        :client="catalog"
        :scope="CatalogScope.EnemySkills"
        :context-key="modelValue.actorKey"
        :selected-key="modelValue.skill?.selection.entryKey ?? null"
        :choice-key="modelValue.skill?.selection.choiceKey ?? null"
        :label="
          t(
            preset === WidgetPreset.EventCountdown
              ? 'modules.interfaces.presets.enemy.afterSkill'
              : 'modules.interfaces.findSkill',
          )
        "
        @choose="skill"
      />
      <template v-if="preset === WidgetPreset.EventCountdown">
        <CatalogPicker
          :key="modelValue.actorKey + ':expected'"
          :client="catalog"
          :scope="CatalogScope.EnemySkills"
          :context-key="modelValue.actorKey"
          :selected-key="modelValue.expectedSkill?.selection.entryKey ?? null"
          :choice-key="modelValue.expectedSkill?.selection.choiceKey ?? null"
          :label="t('modules.interfaces.presets.enemy.expectedSkill')"
          @choose="
            (item, form, version) => update({ expectedSkill: selection(item, form, version) })
          "
        />
        <UiField :label="t('modules.interfaces.presets.enemy.countdownSeconds')">
          <UiNumber
            :model-value="modelValue.countdownSeconds"
            :min="0.1"
            :max="3600"
            :step="0.1"
            :label="t('modules.interfaces.presets.enemy.countdownSeconds')"
            @update:model-value="countdown"
          />
        </UiField>
        <UiField :label="t('modules.interfaces.presets.enemy.warningSeconds')">
          <UiNumber
            :model-value="modelValue.warningSeconds"
            :min="0.1"
            :max="modelValue.countdownSeconds"
            :step="0.1"
            :label="t('modules.interfaces.presets.enemy.warningSeconds')"
            @update:model-value="update({ warningSeconds: $event })"
          />
        </UiField>
        <p class="hint">{{ t('modules.interfaces.presets.enemy.estimateHint') }}</p>
      </template>
      <p v-else class="hint">{{ t('modules.interfaces.rules.castDurationFromEvent') }}</p>
      <UiSwitch
        :model-value="modelValue.sound"
        :label="t('modules.interfaces.presets.enemy.sound')"
        @update:model-value="update({ sound: $event })"
      />
    </template>
  </div>
</template>
