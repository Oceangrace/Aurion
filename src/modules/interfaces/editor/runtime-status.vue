<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { LiveFrame } from '../model/live-frame';
import { SourceStatus } from '../model/source-status';

const props = defineProps<{
  frame: LiveFrame | null;
  error: string | null;
}>();

const { t } = useI18n();

const status = computed(() => props.frame?.status ?? SourceStatus.Offline);

const active = computed(
  () =>
    status.value === SourceStatus.Live &&
    props.frame?.inWorld === true &&
    props.frame.characterDataComplete === true &&
    props.error === null,
);

const failed = computed(() =>
  Object.entries(props.frame?.documentErrors ?? {})
    .map(([name, error]) => `${name}: ${error}`)
    .join(' · '),
);

const label = computed(() => {
  if (props.error !== null) {
    return `${t('modules.interfaces.runtime.error')} · ${props.error}`;
  }

  if (failed.value !== '') {
    return `${t('modules.interfaces.runtime.documentError')} · ${failed.value}`;
  }

  if (status.value === SourceStatus.Live && props.frame?.characterDataComplete !== true) {
    return t('modules.interfaces.runtime.waitingForCharacter');
  }

  return t(`modules.interfaces.runtime.source.${status.value}`);
});
</script>
<template>
  <span
    class="runtime-status"
    :data-state="status"
    :class="{ 'has-error': error !== null || failed !== '', 'is-active': active }"
    :title="label"
    :aria-label="label"
    role="status"
    tabindex="0"
  >
    <span class="status-dot" aria-hidden="true" />
  </span>
</template>
<style scoped>
.runtime-status {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 24px;
  height: 30px;
  color: var(--prism-dim);
  border-radius: var(--prism-radius-control);
}
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}
.runtime-status.is-active {
  color: var(--prism-accent);
}
.runtime-status[data-state='replay'],
.runtime-status[data-state='live']:not(.is-active),
.runtime-status[data-state='stale'] {
  color: #e6b978;
}
.runtime-status.has-error {
  color: #ef807a;
}
.runtime-status:focus-visible {
  outline: 1px solid var(--prism-accent);
}
</style>
