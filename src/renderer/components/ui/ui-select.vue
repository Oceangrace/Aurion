<script setup lang="ts" generic="T extends string">
import { computed, onMounted, onUnmounted, reactive, useId, useTemplateRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { VNode } from 'vue';
import type { UiSelectOption } from './ui-select-option';
import type { UiSelectState } from './ui-select-state';
import { UiSelectController } from './ui-select-controller';
import { UiSelectMenu } from './ui-select-menu';
import type { UiSelectMenuState } from './ui-select-menu-state';
import UiSelectBranch from './ui-select-branch.vue';

const props = withDefaults(
  defineProps<{
    modelValue: T;
    options: readonly UiSelectOption<T>[];
    label: string;
    disabled?: boolean;
    popupMinWidth?: number;
    displayLabel?: string | null;
    compact?: boolean;
    remoteSearch?: boolean;
  }>(),
  { disabled: false, popupMinWidth: 0, displayLabel: null, compact: false, remoteSearch: false },
);

const emit = defineEmits<{ 'update:modelValue': [value: T]; search: [query: string] }>();

const slots = defineSlots<{
  option?(props: { option: UiSelectOption<T> }): VNode[];
  footer?(): VNode[];
}>();

const id = useId();

const { t } = useI18n();

const trigger = useTemplateRef<HTMLButtonElement>('trigger');

const popup = useTemplateRef<HTMLElement>('popup');

const searchInput = useTemplateRef<HTMLInputElement>('searchInput');

const menuState = reactive<UiSelectMenuState>({ query: '', expanded: [], limit: 80 });

const menu = new UiSelectMenu(props, menuState);

const sections = computed(() => menu.sections());

const visibleOptions = computed(() => menu.visible(sections.value));

const indices = computed(
  () => new Map(visibleOptions.value.map((option, index) => [option.value, index])),
);

const state = reactive<UiSelectState>({
  open: false,
  activeIndex: -1,
  left: 0,
  top: 0,
  width: 0,
  maxHeight: 0,
  fontSize: 0,
});

const selectedLabel = computed(
  () =>
    props.options.find((option) => option.value === props.modelValue)?.label ??
    t('select.unavailable'),
);

let controller: UiSelectController<T> | null = null;

onMounted(() => {
  if (trigger.value === null) {
    return;
  }

  controller = new UiSelectController(
    {
      get modelValue() {
        return props.modelValue;
      },
      get options() {
        return visibleOptions.value;
      },
      get allOptions() {
        return props.options;
      },
      get disabled() {
        return props.disabled;
      },
      get popupMinWidth() {
        return props.popupMinWidth;
      },
      get compact() {
        return props.compact;
      },
      get remoteSearch() {
        return props.remoteSearch;
      },
    },
    trigger.value,
    () => popup.value,
    state,
    (value) => {
      emit('update:modelValue', value);
    },
  );
  controller.start();
});
watch(
  [
    () => props.modelValue,
    () =>
      props.options
        .map((option) => [option.value, option.label, option.group, option.parentGroup])
        .join('|'),
    () => props.disabled,
  ],
  () => {
    controller?.sync();
  },
);
watch(
  () => state.open,
  () => {
    if (state.open === false) {
      return;
    }

    menu.reset();
    emit('search', '');
    controller?.sync();
    searchInput.value?.focus({ preventScroll: true });
    requestAnimationFrame(() => controller?.reveal());
  },
  { flush: 'post' },
);

const search = (event: Event): void => {
  if (event.target instanceof HTMLInputElement === false) {
    return;
  }

  menu.search(event.target.value);
  emit('search', event.target.value);
  controller?.sync();
  const list = popup.value?.querySelector('.ui-select-list');

  if (list instanceof HTMLElement === true) {
    list.scrollTop = 0;
  }
};

const searchKeydown = (event: KeyboardEvent): void => {
  if (event.key === 'Tab' && event.shiftKey === false && slots.footer !== undefined) {
    return;
  }

  if (['ArrowDown', 'ArrowUp', 'Enter', 'Escape', 'Tab'].includes(event.key) === true) {
    controller?.keydown(event);
  }
};

onUnmounted(() => {
  controller?.dispose();
});
</script>

<template>
  <span class="ui-select" :class="{ 'ui-select-compact': compact }">
    <button
      ref="trigger"
      type="button"
      class="ui-select-trigger"
      role="combobox"
      :disabled="disabled || (!remoteSearch && options.length === 0)"
      :aria-label="label"
      :title="displayLabel ?? selectedLabel"
      :aria-expanded="state.open"
      aria-haspopup="listbox"
      :aria-controls="state.open ? id : undefined"
      :aria-activedescendant="
        state.open && state.activeIndex !== -1 ? `${id}-${state.activeIndex}` : undefined
      "
      @click="controller?.toggle()"
      @keydown="controller?.keydown($event)"
    >
      <span>{{ displayLabel ?? selectedLabel }}</span>
      <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m6 8 4 4 4-4" /></svg>
    </button>
    <Teleport to=".application-surface" defer>
      <div
        v-if="state.open"
        ref="popup"
        class="ui-select-popup"
        :class="{ 'ui-select-compact-popup': compact }"
        :aria-label="label"
        data-overlay-interactive
        :style="{
          left: `${state.left}px`,
          top: `${state.top}px`,
          width: `${state.width}px`,
          maxHeight: `${state.maxHeight}px`,
          fontSize: `${state.fontSize}px`,
        }"
        @keydown.esc="controller?.close()"
      >
        <input
          ref="searchInput"
          class="ui-input ui-select-search"
          type="search"
          :value="menuState.query"
          :aria-label="t('select.search') + ' · ' + label"
          :placeholder="t('select.search')"
          role="combobox"
          aria-autocomplete="list"
          aria-expanded="true"
          :aria-controls="id"
          :aria-activedescendant="
            state.activeIndex === -1 ? undefined : `${id}-${state.activeIndex}`
          "
          @input="search"
          @keydown="searchKeydown"
        />
        <div :id="id" class="ui-select-list" role="listbox" :aria-label="label">
          <UiSelectBranch
            v-for="section in sections"
            :id="id"
            :key="section.key"
            :section="section"
            :selection="modelValue"
            :indices="indices"
            :active-index="state.activeIndex"
            :query="menuState.query"
            :level="0"
            :label="label"
            @choose="controller?.choose($event)"
            @highlight="state.activeIndex = $event"
            @toggle="
              menu.toggle($event);
              controller?.sync();
            "
            @more="
              menu.more();
              controller?.sync();
            "
            @dismiss="controller?.close()"
          >
            <template #option="{ option }">
              <slot name="option" :option="option"
                ><span>{{ option.label }}</span></slot
              >
            </template>
          </UiSelectBranch>
          <p v-if="sections.length === 0" class="ui-select-empty">{{ t('select.empty') }}</p>
        </div>
        <slot name="footer" />
      </div>
    </Teleport>
  </span>
</template>
