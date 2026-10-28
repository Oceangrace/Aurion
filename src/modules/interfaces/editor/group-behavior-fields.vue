<script setup lang="ts">
import { Direction } from '../model/direction';
import { computed } from 'vue';
import type { CatalogClient } from '../triggers/catalog-client';
import { useI18n } from 'vue-i18n';
import type { WidgetNode } from '../model/widget-node';
import type { EditorSession } from './editor-session';
import type { GroupSettings } from '../model/group-settings';
import type { GroupHeaderButton } from '../model/rules/group-header-button';
import type { GroupDataBinding } from '../model/rules/group-data-binding';
import { defaultGroupDataBinding } from '../model/rules/group-data-binding';
import { GroupSort } from '../model/group-sort';
import { GroupGrow } from '../model/group-grow';
import { TextAlignment } from '../model/text-alignment';
import { GroupHeaderAction } from '../model/rules/group-header-action';
import { GroupHeaderIcon } from '../model/rules/group-header-icon';
import { GroupButtonDisplay } from '../model/rules/group-button-display';
import { GroupButtonPlacement } from '../model/rules/group-button-placement';
import { CombatMetric } from '../model/rules/combat-metric';
import { CombatDirection } from '../model/rules/combat-direction';
import { CombatScope } from '../model/rules/combat-scope';
import { TableDimension } from '../model/rules/table-dimension';
import GroupEffectFields from './group-effect-fields.vue';
import GroupBossFields from './group-boss-fields.vue';
import { defaultGroupEffectSettings } from '../model/rules/group-effect-settings';
import { GroupDataSource } from '../model/rules/group-data-source';
import { InspectorTab } from './inspector-tab';
import { GroupFlow } from '../model/group-flow';
import { WidgetKind } from '../model/widget-kind';
import UiSlider from '../../../renderer/components/ui/ui-slider.vue';
import UiField from '../../../renderer/components/ui/ui-field.vue';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import UiInput from '../../../renderer/components/ui/ui-input.vue';
import UiNumber from '../../../renderer/components/ui/ui-number.vue';
import UiSwitch from '../../../renderer/components/ui/ui-switch.vue';
import UiButton from '../../../renderer/components/ui/ui-button.vue';
import GroupButtonIcon from '../scene/group-button-icon.vue';
const props = defineProps<{
  widget: WidgetNode;
  session: EditorSession;
  tab: InspectorTab;
  catalog: CatalogClient;
}>();

const { t, locale } = useI18n();

const options = <T extends string>(
  values: Readonly<Record<string, T>>,
): { value: T; label: string }[] =>
  Object.values(values).map((value) => ({
    value,
    label: t('modules.interfaces.advanced.options.' + value),
  }));

const gameIconCount = computed(() => Object.keys(props.catalog.icons()).length);

const icons = computed(() => [
  ...options(GroupHeaderIcon),
  ...Object.entries(props.catalog.icons())
    .map(([file, label]) => ({ value: 'game:' + file, label }))
    .sort((left, right) => left.label.localeCompare(right.label, locale.value)),
]);

const settings = (patch: Partial<GroupSettings>): void => {
  props.session.updateSelected((node) => ({ ...node, group: { ...node.group, ...patch } }));
};

const numbers = computed(() => {
  if (props.tab === InspectorTab.Style && props.widget.group.headerEnabled === true) {
    return ['headerFontSize'] as const;
  }

  return props.tab === InspectorTab.Placement
    ? (['limit', 'motionMs', 'spanSeconds'] as const)
    : [];
});

const buttons = (values: readonly GroupHeaderButton[]): void => {
  props.session.updateSelected((node) => ({ ...node, headerButtons: values }));
};

const updateButton = (index: number, patch: Partial<GroupHeaderButton>): void => {
  buttons(
    props.widget.headerButtons.map((entry, at) => (at === index ? { ...entry, ...patch } : entry)),
  );
};

const addButton = (): void => {
  buttons([
    ...props.widget.headerButtons,
    {
      id: crypto.randomUUID(),
      label: '',
      display: GroupButtonDisplay.Icon,
      placement: GroupButtonPlacement.Right,
      action: GroupHeaderAction.Collapse,
      icon: GroupHeaderIcon.Chevron,
    },
  ]);
};

const setBinding = (patch: Partial<GroupDataBinding>): void => {
  props.session.updateSelected((node) => ({
    ...node,
    binding: { ...(node.binding ?? defaultGroupDataBinding), ...patch },
  }));
};

const setSource = (source: GroupDataSource): void => {
  if (source === GroupDataSource.WorldBosses) {
    props.session.updateSelected((node) => ({
      ...node,
      binding: {
        ...(node.binding ?? defaultGroupDataBinding),
        source,
        limit: 48,
        showBreakdown: false,
      },
      group: { ...node.group, flow: GroupFlow.Column },
    }));

    return;
  }

  if (source === GroupDataSource.SkillHistory) {
    props.session.updateSelected((node) => ({
      ...node,
      binding: { ...(node.binding ?? defaultGroupDataBinding), source, limit: 32 },
      group: {
        ...node.group,
        flow: GroupFlow.Timeline,
        spanSeconds: 8,
        travel: Direction.Left,
        motionMs: 100,
      },
    }));

    return;
  }

  if (source !== GroupDataSource.Effects) {
    setBinding({ source });

    return;
  }

  props.session.updateSelected((node) => {
    const previous = node.binding ?? defaultGroupDataBinding;

    const effects = previous.effects ?? null;

    const first = effects === null;

    return {
      ...node,
      binding: {
        ...previous,
        source,
        limit: first ? 128 : previous.limit,
        effects: effects ?? { ...defaultGroupEffectSettings },
      },
      style: first ? { ...node.style, showTimer: true, showStacks: true } : node.style,
      group: {
        ...node.group,
        flow: node.group.flow === GroupFlow.Timeline ? GroupFlow.Grid : node.group.flow,
      },
    };
  });
};

const binding = computed(() => props.widget.binding);

const combatSource = computed(() =>
  [GroupDataSource.Game, GroupDataSource.Preview].includes(
    binding.value?.source ?? GroupDataSource.Game,
  ),
);

const selectors = computed(() =>
  (
    [
      { key: 'source', values: options(GroupDataSource) },
      { key: 'metric', values: options(CombatMetric) },
      { key: 'direction', values: options(CombatDirection) },
      { key: 'dimension', values: options(TableDimension) },
      { key: 'scope', values: options(CombatScope) },
    ] as const
  ).filter((field) => field.key === 'source' || combatSource.value),
);
</script>
<template>
  <UiField
    v-if="tab === InspectorTab.Style && widget.group.headerEnabled"
    :label="t('modules.interfaces.advanced.headerOpacity')"
    ><UiSlider
      :model-value="
        widget.group.headerOpacity ??
        (widget.group.headerBackground.length === 9
          ? Math.round(parseInt(widget.group.headerBackground.slice(7), 16) / 2.55)
          : 100)
      "
      :label="t('modules.interfaces.advanced.headerOpacity')"
      @update:model-value="
        settings({
          headerOpacity: $event,
          headerBackground:
            widget.group.headerBackground.slice(0, 7) +
            Math.round($event * 2.55)
              .toString(16)
              .padStart(2, '0'),
        })
      "
  /></UiField>
  <div v-if="numbers.length > 0" class="field-grid">
    <UiField
      v-if="tab === InspectorTab.Style"
      :label="t('modules.interfaces.advanced.headerAlignment')"
      ><UiSelect
        :model-value="widget.group.headerAlignment"
        :options="options(TextAlignment)"
        :label="t('modules.interfaces.advanced.headerAlignment')"
        @update:model-value="settings({ headerAlignment: $event })"
    /></UiField>
    <UiField v-for="key in numbers" :key="key" :label="t('modules.interfaces.advanced.' + key)"
      ><UiNumber
        :model-value="widget.group[key]"
        :label="t('modules.interfaces.advanced.' + key)"
        :min="key === 'headerFontSize' ? 6 : key === 'spanSeconds' ? 0.1 : 0"
        :max="key === 'headerFontSize' ? 96 : key === 'limit' ? 128 : 3600"
        @update:model-value="settings({ [key]: $event })"
    /></UiField>
  </div>
  <template v-if="widget.kind === WidgetKind.DynamicGroup && tab === InspectorTab.Placement">
    <UiField :label="t('modules.interfaces.advanced.sort')"
      ><UiSelect
        :model-value="widget.group.sort"
        :options="options(GroupSort)"
        :label="t('modules.interfaces.advanced.sort')"
        @update:model-value="settings({ sort: $event })"
    /></UiField>
    <UiField :label="t('modules.interfaces.advanced.grow')"
      ><UiSelect
        :model-value="widget.group.grow"
        :options="options(GroupGrow)"
        :label="t('modules.interfaces.advanced.grow')"
        @update:model-value="settings({ grow: $event })"
    /></UiField>
    <UiField :label="t('modules.interfaces.advanced.travel')"
      ><UiSelect
        :model-value="widget.group.travel"
        :options="
          [Direction.Left, Direction.Right, Direction.Up, Direction.Down].map((value) => ({
            value,
            label: t('modules.interfaces.advanced.options.' + value),
          }))
        "
        :label="t('modules.interfaces.advanced.travel')"
        @update:model-value="settings({ travel: $event })"
    /></UiField>
  </template>
  <section
    v-if="widget.kind === WidgetKind.DynamicGroup && tab === InspectorTab.Display"
    class="inspector-card"
  >
    <h3>{{ t('modules.interfaces.advanced.binding') }}</h3>
    <UiSwitch
      :model-value="binding !== null"
      :label="t('modules.interfaces.advanced.binding')"
      @update:model-value="
        session.updateSelected((node) => ({
          ...node,
          binding: $event ? { ...defaultGroupDataBinding } : null,
        }))
      "
    />
    <template v-if="binding !== null">
      <UiField
        v-for="field in selectors"
        :key="field.key"
        :label="t('modules.interfaces.advanced.' + field.key)"
        ><UiSelect
          :model-value="binding[field.key]"
          :options="field.values"
          :label="t('modules.interfaces.advanced.' + field.key)"
          @update:model-value="
            field.key === 'source'
              ? setSource($event as GroupDataSource)
              : setBinding({ [field.key]: $event })
          "
      /></UiField>
      <UiField :label="t('modules.interfaces.advanced.limit')"
        ><UiNumber
          :model-value="binding.limit"
          :label="t('modules.interfaces.advanced.limit')"
          :min="1"
          :max="128"
          @update:model-value="setBinding({ limit: $event })"
      /></UiField>
      <GroupEffectFields
        v-if="binding.source === GroupDataSource.Effects"
        :widget="widget"
        :session="session"
      />
      <GroupBossFields
        v-else-if="binding.source === GroupDataSource.WorldBosses"
        :widget="widget"
        :session="session"
        :catalog="catalog"
      />
      <template v-else-if="combatSource">
        <UiField
          v-for="key in ['actorId', 'targetId'] as const"
          :key="key"
          :label="t('modules.interfaces.advanced.' + key)"
          ><UiInput
            :model-value="binding[key] ?? ''"
            :label="t('modules.interfaces.advanced.' + key)"
            @update:model-value="setBinding({ [key]: $event === '' ? null : $event })"
        /></UiField>
        <UiSwitch
          :model-value="binding.showBreakdown"
          :label="t('modules.interfaces.advanced.showBreakdown')"
          @update:model-value="setBinding({ showBreakdown: $event })"
        />
        <UiSwitch
          :model-value="binding.includeSummons"
          :label="t('modules.interfaces.advanced.includeSummons')"
          @update:model-value="setBinding({ includeSummons: $event })"
        />
      </template>
    </template>
  </section>
  <section v-if="tab === InspectorTab.Display" class="inspector-card">
    <h3>{{ t('modules.interfaces.advanced.headerButtons') }}</h3>
    <div
      v-for="(button, index) in widget.headerButtons"
      :key="button.id"
      class="group-button-entry"
    >
      <header class="group-button-heading">
        <span>{{ t('modules.interfaces.advanced.buttonNumber', { n: index + 1 }) }}</span>
        <UiButton
          :danger="true"
          :disabled="session.isReadOnly()"
          @click="buttons(widget.headerButtons.filter((_, at) => at !== index))"
          >{{ t('modules.interfaces.remove') }}</UiButton
        >
      </header>
      <UiField :label="t('modules.interfaces.text')">
        <UiInput
          :model-value="button.label"
          :label="t('modules.interfaces.text')"
          :disabled="session.isReadOnly()"
          @update:model-value="updateButton(index, { label: $event })"
        />
      </UiField>
      <div class="field-grid">
        <UiField :label="t('modules.interfaces.advanced.action')">
          <UiSelect
            :model-value="button.action"
            :options="options(GroupHeaderAction)"
            :label="t('modules.interfaces.advanced.action')"
            :disabled="session.isReadOnly()"
            @update:model-value="updateButton(index, { action: $event })"
          />
        </UiField>
        <UiField :label="t('modules.interfaces.create.sections.display')">
          <UiSelect
            :model-value="button.display"
            :options="options(GroupButtonDisplay)"
            :label="t('modules.interfaces.create.sections.display')"
            :disabled="session.isReadOnly()"
            @update:model-value="updateButton(index, { display: $event })"
          />
        </UiField>
        <UiField :label="t('modules.interfaces.tabs.placement')">
          <UiSelect
            :model-value="button.placement"
            :options="options(GroupButtonPlacement)"
            :label="t('modules.interfaces.tabs.placement')"
            :disabled="session.isReadOnly()"
            @update:model-value="updateButton(index, { placement: $event })"
          />
        </UiField>
        <UiField
          v-if="button.display !== GroupButtonDisplay.Text"
          :label="t('modules.interfaces.icon')"
          :hint="t('modules.interfaces.advanced.gameIconCount', { n: gameIconCount })"
        >
          <div class="group-icon-field">
            <GroupButtonIcon :icon="button.icon" />
            <UiSelect
              :model-value="button.icon"
              :options="icons"
              :label="t('modules.interfaces.icon')"
              :disabled="session.isReadOnly()"
              @update:model-value="updateButton(index, { icon: $event })"
            >
              <template #option="{ option }">
                <span class="group-icon-option">
                  <GroupButtonIcon :icon="option.value" />
                  <span>{{ option.label }}</span>
                </span>
              </template>
            </UiSelect>
          </div>
        </UiField>
      </div>
    </div>
    <UiButton
      class="group-add-button is-add"
      :disabled="widget.headerButtons.length >= 6 || session.isReadOnly()"
      @click="addButton"
      >＋ {{ t('modules.interfaces.advanced.addButton') }}</UiButton
    >
  </section>
</template>

<style scoped>
.group-button-entry {
  display: grid;
  gap: 14px;
  min-width: 0;
  padding: 16px;
  border: 1px solid var(--prism-border);
  border-radius: 4px;
}
.group-button-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  color: var(--prism-muted);
  font-size: 12px;
}
.group-button-heading .ui-button {
  min-height: 24px;
  padding: 0 6px;
  border-color: transparent;
  background: transparent;
  font-size: 12px;
}
.group-icon-field,
.group-icon-option {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.group-icon-field > .group-button-icon,
.group-icon-option > .group-button-icon {
  font-size: 26px;
}
.group-icon-field > .ui-select {
  min-width: 0;
}
.group-icon-option > span:last-child {
  min-width: 0;
  overflow-wrap: anywhere;
}
.group-add-button {
  align-self: flex-start;
}
</style>
