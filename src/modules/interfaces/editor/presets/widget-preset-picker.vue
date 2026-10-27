<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { CatalogClient } from '../../triggers/catalog-client';
import type { CatalogItem } from '../../model/catalog-item';
import type { CatalogForm } from '../../model/catalog-form';
import type { WidgetNode } from '../../model/widget-node';
import type { WidgetKind } from '../../model/widget-kind';
import type { WidgetPresetSelection } from './widget-preset-selection';
import { CatalogScope } from '../../model/catalog-scope';
import { CatalogSource } from '../../model/catalog-source';
import { TriggerUnit } from '../../model/rules/trigger-unit';
import { WidgetPreset } from './widget-preset';
import { WidgetPresetCatalog } from './widget-preset-catalog';
import { WidgetPresetFactory } from './widget-preset-factory';
import { defaultEnemyWidgetPresetOptions } from './enemy-widget-preset-options';
import EnemyPresetFields from './enemy-preset-fields.vue';
import CatalogPicker from '../../triggers/catalog-picker.vue';
import WidgetPresetPreview from './widget-preset-preview.vue';
import UiButton from '../../../../renderer/components/ui/ui-button.vue';
import UiField from '../../../../renderer/components/ui/ui-field.vue';
import UiSelect from '../../../../renderer/components/ui/ui-select.vue';
import UiSwitch from '../../../../renderer/components/ui/ui-switch.vue';
import './widget-presets.css';

const props = defineProps<{ kind: WidgetKind; catalog: CatalogClient; disabled: boolean }>();

const emit = defineEmits<{ create: [widget: WidgetNode] }>();

const { t } = useI18n();

const presets = new WidgetPresetCatalog();

const factory = new WidgetPresetFactory();

const preset = ref(WidgetPreset.Empty);

const selected = ref<WidgetPresetSelection | null>(null);

const unit = ref<TriggerUnit.Player | TriggerUnit.Target>(TriggerUnit.Player);

const showTimer = ref(true);

const enemy = ref({ ...defaultEnemyWidgetPresetOptions });

const source = ref(CatalogSource.ClassSkills);

const sources = computed(() =>
  Object.values(CatalogSource).map((value) => ({
    value,
    label: t('modules.interfaces.catalogSources.' + value),
  })),
);

const scope = computed(() => presets.scope(preset.value));

const choices = computed(() => presets.list(props.kind));

const units = computed(() =>
  [TriggerUnit.Player, TriggerUnit.Target].map((value) => ({
    value,
    label: t('modules.interfaces.rules.options.' + value),
  })),
);

const draft = computed(() =>
  factory.create(
    props.kind,
    preset.value,
    t(
      preset.value === WidgetPreset.Empty
        ? 'modules.interfaces.types.' + props.kind
        : 'modules.interfaces.presets.names.' + preset.value,
    ),
    { unit: unit.value, showTimer: showTimer.value, catalog: selected.value, enemy: enemy.value },
  ),
);

const select = (value: WidgetPreset): void => {
  preset.value = value;
  selected.value = null;
  unit.value = TriggerUnit.Player;
  showTimer.value = value !== WidgetPreset.Ready;
  enemy.value = { ...defaultEnemyWidgetPresetOptions };
};

const choose = (item: CatalogItem, form: CatalogForm, version: string): void => {
  const currentScope = scope.value;

  if (currentScope === null) {
    return;
  }

  selected.value = {
    item,
    selection: {
      catalogVersion: version,
      scope: currentScope,
      contextKey: null,
      entryKey: item.key,
      choiceKey: form.key,
    },
  };
};

const changeSource = (value: CatalogSource): void => {
  source.value = value;
  selected.value = null;
};

const create = (): void => {
  if (draft.value !== null && props.disabled === false) {
    emit('create', draft.value);
  }
};
</script>
<template>
  <div class="element-preset-picker">
    <div class="element-preset-choices" :aria-label="t('modules.interfaces.presets.title')">
      <button
        v-for="value in choices"
        :key="value"
        type="button"
        class="element-preset-choice"
        :aria-pressed="preset === value"
        :disabled="disabled"
        @click="select(value)"
      >
        <svg viewBox="0 0 20 20" aria-hidden="true">
          <path v-if="value === WidgetPreset.Empty" d="M10 4v12 M4 10h12" />
          <path v-else-if="preset === value" d="m4 10 4 4 8-8" />
          <circle v-else cx="10" cy="10" r="5" />
        </svg>
        <span class="element-copy">
          <strong>{{ t('modules.interfaces.presets.names.' + value) }}</strong>
          <small>{{ t('modules.interfaces.presets.descriptions.' + value) }}</small>
        </span>
      </button>
    </div>
    <div class="element-preset-settings">
      <WidgetPresetPreview :widget="draft" :preset="preset" :kind="kind" />
      <EnemyPresetFields
        v-if="presets.isEnemy(preset)"
        v-model="enemy"
        :preset="preset"
        :catalog="props.catalog"
        :disabled="disabled"
      />
      <UiField v-if="scope === CatalogScope.Effects" :label="t('modules.interfaces.scope')">
        <UiSelect
          :model-value="source"
          :options="sources"
          :label="t('modules.interfaces.scope')"
          :disabled="disabled"
          @update:model-value="changeSource"
        />
      </UiField>
      <CatalogPicker
        v-if="scope !== null && !disabled"
        :key="preset + ':' + source"
        :client="props.catalog"
        :scope="scope"
        :context-key="null"
        :category="presets.effectFilter(preset)"
        :source="scope === CatalogScope.Effects ? source : CatalogSource.All"
        :grouped="true"
        :selected-key="selected?.selection.entryKey ?? null"
        :choice-key="selected?.selection.choiceKey ?? null"
        :label="
          t(
            scope === CatalogScope.Effects
              ? 'modules.interfaces.findEffect'
              : 'modules.interfaces.findSkill',
          )
        "
        @choose="choose"
      />
      <UiField v-if="presets.selectableUnit(preset)" :label="t('modules.interfaces.rules.unit')">
        <UiSelect
          v-model="unit"
          :options="units"
          :label="t('modules.interfaces.rules.unit')"
          :disabled="disabled"
        />
      </UiField>
      <UiSwitch
        v-if="scope !== null || presets.isEnemy(preset)"
        v-model="showTimer"
        :label="t('modules.interfaces.presets.showTimer')"
        :disabled="disabled"
      />
      <div class="element-preset-actions">
        <UiButton class="is-primary" :disabled="disabled || draft === null" @click="create">{{
          t('modules.interfaces.create.elementTitle')
        }}</UiButton>
      </div>
    </div>
  </div>
</template>
