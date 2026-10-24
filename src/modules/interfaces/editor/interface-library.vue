<script setup lang="ts">
import { HudLocalization } from '../scene/hud-localization';
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { EditorSession } from './editor-session';
import UiButton from '../../../renderer/components/ui/ui-button.vue';
import UiIconButton from '../../../renderer/components/ui/ui-icon-button.vue';
import UiConfirmDialog from '../../../renderer/components/ui/ui-confirm-dialog.vue';
import UiMessage from '../../../renderer/components/ui/ui-message.vue';
import type { UnavailableProfile } from '../model/unavailable-profile';

const props = defineProps<{
  session: EditorSession;
  unavailable: readonly UnavailableProfile[];
}>();

const emit = defineEmits<{ open: [id: string]; create: []; removeUnavailable: [folder: string] }>();

const { t, locale } = useI18n();

const localization = new HudLocalization();

const query = ref('');

const state = props.session.getState();

const documents = computed(() =>
  state.value.workspace.documents
    .map((entry) => ({
      ...entry,
      name: localization.text(entry.name, entry.localization, locale.value),
      description: localization.text(entry.description, entry.localization, locale.value),
    }))
    .filter((entry) =>
      (entry.name + ' ' + entry.description + ' ' + entry.author)
        .toLocaleLowerCase()
        .includes(query.value.toLocaleLowerCase()),
    ),
);

const removalId = ref<string | null>(null);

const removing = computed(
  () => state.value.workspace.documents.find((entry) => entry.id === removalId.value) ?? null,
);

watch(removing, (entry) => {
  if (entry === null) {
    removalId.value = null;
  }
});

const unavailableRemoval = ref<string | null>(null);

const confirmUnavailableRemove = (): void => {
  const folder = unavailableRemoval.value;

  unavailableRemoval.value = null;

  if (folder !== null) {
    emit('removeUnavailable', folder);
  }
};

const confirmRemove = (): void => {
  const entry = removing.value;

  removalId.value = null;

  if (entry === null || props.session.isReadOnly() === true) {
    return;
  }

  props.session.selectDocument(entry.id);
  props.session.removeDocument();
};
</script>
<template>
  <section class="interface-library editor-page">
    <div class="library-tools">
      <input
        v-model="query"
        class="ui-input"
        type="search"
        :placeholder="t('modules.interfaces.redesign.searchLibrary')"
        :aria-label="t('modules.interfaces.redesign.searchLibrary')"
      />
      <UiButton :disabled="state.readOnly" @click="emit('create')">
        <span aria-hidden="true">＋</span>{{ t('modules.interfaces.newInterface') }}
      </UiButton>
    </div>
    <div class="library-table" role="table" :aria-label="t('modules.interfaces.interfaces')">
      <div class="library-row library-columns" role="row">
        <span role="columnheader">{{ t('modules.interfaces.name') }}</span>
        <span role="columnheader">{{ t('modules.interfaces.information.author') }}</span>
        <span role="columnheader">{{ t('modules.interfaces.redesign.version') }}</span>
        <span role="columnheader">{{ t('modules.interfaces.redesign.composition') }}</span>
        <span role="columnheader" class="visually-hidden">{{
          t('modules.interfaces.redesign.open')
        }}</span>
      </div>
      <div v-for="entry in documents" :key="entry.id" class="library-row" role="row">
        <div role="cell" class="library-name">
          <button type="button" @click="emit('open', entry.id)">{{ entry.name }}</button>
          <small v-if="entry.description">{{ entry.description }}</small>
        </div>
        <span role="cell">{{ entry.author || '—' }}</span>
        <span role="cell">{{ entry.version || '—' }}</span>
        <span role="cell" class="library-counts">
          {{
            t('modules.interfaces.information.elementCount', {
              n: session.tree.flatten(entry.roots).length,
            })
          }}
          <small>{{
            t('modules.interfaces.information.formulaCount', { n: entry.formulas.length })
          }}</small>
        </span>
        <span role="cell" class="library-row-actions"
          ><UiButton @click="emit('open', entry.id)">{{
            t('modules.interfaces.redesign.open')
          }}</UiButton>
          <UiIconButton
            :label="t('modules.interfaces.information.remove') + ' · ' + entry.name"
            :disabled="state.readOnly"
            @click="removalId = entry.id"
            ><svg viewBox="0 0 20 20" aria-hidden="true">
              <path d="M4 6h12 M7 6V3h6v3 M5 6l1 11h8l1-11 M8 9v5 M12 9v5" /></svg
          ></UiIconButton>
        </span>
      </div>
      <div
        v-for="entry in unavailable"
        :key="'unavailable:' + entry.folder"
        class="library-row library-unavailable"
        role="row"
      >
        <div role="cell" class="library-name">
          <span>{{ entry.folder }}</span>
          <small>{{ t('modules.interfaces.unavailable.title') }}</small>
        </div>
        <span role="cell">—</span>
        <span role="cell">—</span>
        <span role="cell" class="library-counts"><UiMessage :text="entry.reason" /></span>
        <span role="cell" class="library-row-actions"
          ><UiIconButton
            :label="t('modules.interfaces.information.remove') + ' · ' + entry.folder"
            :disabled="state.readOnly"
            @click="unavailableRemoval = entry.folder"
            ><svg viewBox="0 0 20 20" aria-hidden="true">
              <path d="M4 6h12 M7 6V3h6v3 M5 6l1 11h8l1-11 M8 9v5 M12 9v5" /></svg
          ></UiIconButton>
        </span>
      </div>
    </div>
    <p v-if="documents.length === 0 && unavailable.length === 0" class="hint">
      {{ t('modules.interfaces.emptyFilter') }}
    </p>
    <p v-if="unavailable.length > 0" class="hint">{{ t('modules.interfaces.unavailable.hint') }}</p>
    <UiConfirmDialog
      :open="removing !== null"
      :title="t('modules.interfaces.confirmRemove', { name: removing?.name ?? '' })"
      :message="t('modules.interfaces.redesign.deleteMessage')"
      :cancel-label="t('modules.interfaces.cancel')"
      :confirm-label="t('modules.interfaces.confirm')"
      @cancel="removalId = null"
      @confirm="confirmRemove"
    />
    <UiConfirmDialog
      :open="unavailableRemoval !== null"
      :title="t('modules.interfaces.confirmRemove', { name: unavailableRemoval ?? '' })"
      :message="t('modules.interfaces.unavailable.deleteMessage')"
      :cancel-label="t('modules.interfaces.cancel')"
      :confirm-label="t('modules.interfaces.confirm')"
      @cancel="unavailableRemoval = null"
      @confirm="confirmUnavailableRemove"
    />
  </section>
</template>
