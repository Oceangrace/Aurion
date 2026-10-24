<script setup lang="ts">
import { GroupButtonPlacement } from '../model/rules/group-button-placement';
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { WidgetNode } from '../model/widget-node';
import type { EditorSession } from '../editor/editor-session';
import { GroupHeaderAction } from '../model/rules/group-header-action';
import { CombatMetric } from '../model/rules/combat-metric';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import GroupButtonIcon from './group-button-icon.vue';
const props = defineProps<{ widget: WidgetNode; session: EditorSession; collapsed: boolean }>();

const buttons = (placement: GroupButtonPlacement): WidgetNode['headerButtons'] =>
  props.widget.headerButtons.filter((button) => button.placement === placement);

const emit = defineEmits<{ collapse: []; edit: [] }>();

const { t } = useI18n();

const menu = ref<GroupHeaderAction | null>(null);

const metrics = computed(() =>
  Object.values(CombatMetric).map((value) => ({
    value,
    label: t('modules.interfaces.advanced.options.' + value),
  })),
);

const changeMetric = (metric: CombatMetric): void => {
  props.session.selectWidget(props.widget.id);
  props.session.updateSelected((node) => ({
    ...node,
    binding: node.binding === null ? null : { ...node.binding, metric },
  }));
  menu.value = null;
  void props.session.save();
};

const cycleMetric = (): void => {
  const values = Object.values(CombatMetric);

  const current = props.widget.binding?.metric ?? CombatMetric.Damage;

  const index = values.indexOf(current);

  changeMetric(values[(index + 1) % values.length] ?? CombatMetric.Damage);
};

const action = (value: GroupHeaderAction): void => {
  props.session.selectWidget(props.widget.id);

  if (value === GroupHeaderAction.Edit) {
    emit('edit');

    return;
  }

  if (value === GroupHeaderAction.Collapse) {
    emit('collapse');

    return;
  }

  if (value === GroupHeaderAction.Close) {
    props.session.toggleWidget(props.widget.id);
    void props.session.save();

    return;
  }

  if (value === GroupHeaderAction.CycleMetric) {
    cycleMetric();

    return;
  }

  menu.value = menu.value === value ? null : value;
};

const panels = computed(() =>
  props.session.tree
    .flatten(props.session.document()?.roots ?? [])
    .filter((node) => props.session.tree.isGroup(node)),
);
</script>
<template>
  <header
    class="widget-group-header"
    :style="{
      height: String(widget.group.headerHeight) + 'px',
      fontSize: String(widget.group.headerFontSize) + 'px',
      color: widget.group.headerColor,
      background: widget.group.headerBackground,
      textAlign: widget.group.headerAlignment,
    }"
  >
    <span class="header-buttons"
      ><button
        v-for="button in buttons(GroupButtonPlacement.Left)"
        :key="button.id"
        type="button"
        data-overlay-interactive
        :aria-label="button.label || t('modules.interfaces.advanced.options.' + button.action)"
        @pointerdown.stop
        @click.stop="action(button.action)"
      >
        <GroupButtonIcon
          v-if="button.display !== 'text'"
          :icon="button.icon"
          :class="{ 'is-collapsed': button.action === GroupHeaderAction.Collapse && collapsed }"
        />
        <span v-if="button.display !== 'icon'">{{ button.label }}</span>
      </button></span
    >
    <span class="header-title">{{ widget.group.headerText || widget.name }}</span>
    <span class="header-buttons"
      ><button
        v-for="button in buttons(GroupButtonPlacement.Right)"
        :key="button.id"
        type="button"
        data-overlay-interactive
        :aria-label="button.label || t('modules.interfaces.advanced.options.' + button.action)"
        @pointerdown.stop
        @click.stop="action(button.action)"
      >
        <GroupButtonIcon
          v-if="button.display !== 'text'"
          :icon="button.icon"
          :class="{ 'is-collapsed': button.action === GroupHeaderAction.Collapse && collapsed }"
        />
        <span v-if="button.display !== 'icon'">{{ button.label }}</span>
      </button></span
    >
    <div v-if="menu !== null" class="group-header-menu" data-overlay-interactive @pointerdown.stop>
      <UiSelect
        v-if="menu === GroupHeaderAction.SelectMetric && widget.binding !== null"
        :model-value="widget.binding.metric"
        :options="metrics"
        :label="t('modules.interfaces.advanced.metric')"
        @update:model-value="changeMetric($event)"
      />
      <template v-else
        ><button
          v-for="panel in panels"
          :key="panel.id"
          type="button"
          @click="session.toggleWidget(panel.id)"
        >
          {{ panel.enabled ? '☑' : '☐' }} {{ panel.name }}
        </button></template
      >
    </div>
  </header>
</template>
<style scoped>
.widget-group-header {
  position: absolute;
  inset: 0 0 auto;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 6px;
  pointer-events: auto;
}
.header-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.header-buttons {
  display: flex;
  gap: 2px;
}
.header-buttons button,
.group-header-menu button {
  color: inherit;
  background: #14141480;
  border: 1px solid #444;
  padding: 1px 4px;
  font: inherit;
  cursor: pointer;
}
.header-buttons button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.group-header-menu {
  position: absolute;
  right: 0;
  top: 100%;
  min-width: 180px;
  max-height: 320px;
  overflow: auto;
  z-index: 200;
  background: var(--prism-background);
  padding: 6px;
  display: grid;
  gap: 3px;
}
.group-button-icon.is-collapsed {
  transform: rotate(-90deg);
}
</style>
