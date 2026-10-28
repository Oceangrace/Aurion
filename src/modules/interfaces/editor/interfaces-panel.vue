<script setup lang="ts">
import UiPageHeader from '../../../renderer/components/ui/ui-page-header.vue';
import UiMessage from '../../../renderer/components/ui/ui-message.vue';
import PreviewControls from './preview-controls.vue';
import CharacterStatus from './character-status.vue';
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { InterfacesModule } from '../interfaces-module';
import InterfaceLibrary from './interface-library.vue';
import UiSelect from '../../../renderer/components/ui/ui-select.vue';
import FormulasEditor from './formulas-editor.vue';
import WidgetTreeNode from './widget-tree-node.vue';
import WidgetInspector from './widget-inspector.vue';
import NewElementPage from './new-element-page.vue';
import ImportPage from './import-page.vue';
import InterfaceInformation from './interface-information.vue';
import InterfaceTranslations from './interface-translations.vue';
import ExportPage from './export-page.vue';
import { EditorPage } from './editor-page';
import { LayersLayout } from './layers-layout';
import { LayersPlacement } from './layers-placement';
import LayersPanel from './layers-panel.vue';
import UiButton from '../../../renderer/components/ui/ui-button.vue';
import UiIconButton from '../../../renderer/components/ui/ui-icon-button.vue';
import FloatingSurface from '../../../renderer/shell/floating-surface.vue';
import { InspectorTab } from './inspector-tab';
import { InterfaceTab } from './interface-tab';
import './editor-layout.css';
import { HudLocalization } from '../scene/hud-localization';
const props = defineProps<{ interfaces: InterfacesModule }>();

const emit = defineEmits<{ close: [] }>();

const { t, locale } = useI18n();

const localization = new HudLocalization();

const session = props.interfaces.session;

const state = session.getState();

const snapshot = props.interfaces.getSnapshot();

const liveFrame = props.interfaces.live.getFrame();

const liveError = props.interfaces.live.getError();

const previewEnabled = props.interfaces.preview.enabled;

const query = ref('');

const document = computed(() => session.document());

const selected = computed(() => session.selected());

const documents = computed(() =>
  state.value.workspace.documents
    .map((entry) => localization.document(entry, locale.value))
    .filter((entry) =>
      (
        entry.name +
        ' ' +
        session.tree
          .flatten(entry.roots)
          .map((node) => node.name)
          .join(' ')
      )
        .toLowerCase()
        .includes(query.value.toLowerCase()),
    ),
);

const page = ref(EditorPage.Interface);

const layersOpen = ref(true);

const layersLayout = new LayersLayout(window.localStorage, (error) => {
  session.report(error);
});

layersLayout.load();

const layersState = layersLayout.getState();

const dockTarget = ref<HTMLElement | null>(null);

const docked = computed(
  () => layersOpen.value === true && layersState.value.placement !== LayersPlacement.Floating,
);

const layersRoot = ref<HTMLElement | null>(null);

const rail = ref<HTMLElement | null>(null);

const path = computed(() =>
  state.value.selectedId === null || document.value === null
    ? []
    : session.tree.path(
        localization.document(document.value, locale.value).roots,
        state.value.selectedId,
      ),
);

const documentOptions = computed(() =>
  state.value.workspace.documents.map((entry) => ({
    value: entry.id,
    label: localization.text(entry.name, entry.localization, locale.value),
  })),
);

const closeLayers = (): void => {
  layersOpen.value = false;
  void nextTick(() => rail.value?.querySelector<HTMLElement>('[data-layers-trigger]')?.focus());
};

const openWidget = (documentId: string, id: string, additive = false): void => {
  if (state.value.documentId !== documentId) {
    session.selectDocument(documentId);
  }

  session.select(id, additive);
  page.value = EditorPage.Widget;
};

const createInterface = (): void => {
  session.createDocument(t('modules.interfaces.defaultInterface'));
  page.value = EditorPage.Interface;
};

watch(layersOpen, (open) => {
  if (open === true) {
    void nextTick(() => layersRoot.value?.querySelector<HTMLInputElement>('input')?.focus());
  }
});

const inspectorTab = ref(InspectorTab.Display);

const interfaceTab = ref(InterfaceTab.Information);

const creationParent = ref<string | null>(null);

const exportCurrent = (): void => {
  if (document.value === null) {
    return;
  }

  session.select(null);
  interfaceTab.value = InterfaceTab.Export;
  page.value = EditorPage.Interface;
};

const addElement = (): void => {
  creationParent.value = session.tree.creationParent(
    document.value?.roots ?? [],
    state.value.selectedId,
  );
  session.select(null);
  page.value = EditorPage.Create;
};

const finishCreation = (): void => {
  inspectorTab.value = InspectorTab.Display;
  page.value = selected.value === null ? EditorPage.Interface : EditorPage.Widget;
};

const openInterface = (id: string): void => {
  session.selectDocument(id);
  page.value = EditorPage.Interface;
};

watch(
  () => state.value.selectedId,
  (id) => {
    if (id !== null) {
      page.value = EditorPage.Widget;
      const current = document.value;

      if (current !== null) {
        session.layers.setExpanded(current.id, null, true);
        session.tree
          .path(current.roots, id)
          .slice(0, -1)
          .forEach((node) => {
            session.layers.setExpanded(current.id, node.id, true);
          });
      }
    }
  },
);
watch(
  () => state.value.documentId,
  (id) => {
    if (id === null) {
      page.value = EditorPage.Library;
    }
  },
  { immediate: true },
);
</script>
<template>
  <FloatingSurface
    class="interfaces-surface"
    :title="t('modules.interfaces.title')"
    :x="24"
    :y="80"
    :width="1160"
    :height="800"
    @close="emit('close')"
  >
    <template #leading>
      <span class="editor-brand"
        ><img src="../../../renderer/assets/aurion.svg" width="32" height="32" alt="Aurion"
      /></span>
    </template>
    <template #title>
      <nav
        v-if="page !== EditorPage.Library && page !== EditorPage.Import"
        class="editor-path"
        :aria-label="t('modules.interfaces.interfaceSettings')"
      >
        <template v-if="document !== null">
          <UiSelect
            :model-value="document.id"
            :options="documentOptions"
            :compact="true"
            :label="t('modules.interfaces.interfaces')"
            @update:model-value="openInterface($event)"
          />
          <template v-for="(node, index) in path" :key="node.id">
            <span class="path-separator" aria-hidden="true">/</span>
            <button
              type="button"
              :aria-current="index === path.length - 1 ? 'page' : undefined"
              @click="openWidget(document.id, node.id)"
            >
              {{ node.name }}
            </button>
          </template>
        </template>
        <span v-else>{{ t('modules.interfaces.title') }}</span>
      </nav>
      <span v-else class="editor-path" aria-hidden="true" />
    </template>
    <template #toolbar>
      <div class="editor-top-actions">
        <UiIconButton
          :label="t('modules.interfaces.resetPositions')"
          :disabled="document === null || state.readOnly"
          @click="session.resetPositions()"
        >
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path d="M3 7V3h4 M13 3h4v4 M17 13v4h-4 M7 17H3v-4" />
            <circle cx="10" cy="10" r="2" />
          </svg>
        </UiIconButton>
        <UiIconButton
          :label="t('modules.interfaces.undo')"
          :disabled="state.canUndo === false"
          @click="session.undo()"
        >
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path d="m7 4-4 4 4 4 M3 8h8a5 5 0 0 1 0 10" />
          </svg>
        </UiIconButton>
        <UiIconButton
          :label="t('modules.interfaces.redo')"
          :disabled="state.canRedo === false"
          @click="session.redo()"
        >
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path d="m13 4 4 4-4 4 M17 8h-8a5 5 0 0 0 0 10" />
          </svg>
        </UiIconButton>
        <UiButton
          class="editor-export"
          :disabled="document === null || state.readOnly"
          @click="exportCurrent()"
          >{{ t('modules.interfaces.import.export') }}</UiButton
        >
        <UiButton
          class="is-primary editor-save-button"
          :data-state="state.saveStatus"
          :title="t('modules.interfaces.status.' + state.saveStatus)"
          :aria-label="
            t('modules.interfaces.save') +
            ' · ' +
            t('modules.interfaces.status.' + state.saveStatus)
          "
          :disabled="state.readOnly"
          @click="void session.save()"
        >
          {{ t('modules.interfaces.save') }}
          <span class="visually-hidden" role="status">{{
            t('modules.interfaces.status.' + state.saveStatus)
          }}</span>
        </UiButton>
      </div>
    </template>
    <main class="interfaces-editor">
      <slot name="notices" />
      <CharacterStatus v-if="!previewEnabled" :frame="liveFrame" :error="liveError" />
      <PreviewControls :preview="interfaces.preview" />
      <div v-if="state.message !== null" class="editor-error" role="alert">
        <UiMessage :text="state.message" />
      </div>
      <div
        class="editor-body"
        :class="{ 'has-docked-layers': docked }"
        :data-layers-placement="layersState.placement"
        :style="{ '--layers-width': layersState.width + 'px' }"
      >
        <nav ref="rail" class="editor-rail" :aria-label="t('modules.interfaces.title')">
          <UiIconButton
            :label="t('modules.interfaces.interfaces')"
            :active="page === EditorPage.Library || page === EditorPage.Import"
            @click="page = EditorPage.Library"
          >
            <svg viewBox="0 0 24 24"><path d="M5 5h14v16H5z M8 2h10 M2 8v10" /></svg>
          </UiIconButton>
          <UiIconButton
            data-layers-trigger
            :label="t('modules.interfaces.layers')"
            :active="layersOpen"
            :aria-expanded="layersOpen"
            @click="layersOpen = !layersOpen"
          >
            <svg viewBox="0 0 24 24">
              <path d="m3 8 9-5 9 5-9 5-9-5 M3 12l9 5 9-5 M3 16l9 5 9-5" />
            </svg>
          </UiIconButton>
          <UiIconButton
            :label="t('modules.interfaces.create.elementTitle')"
            :active="page === EditorPage.Create"
            :disabled="state.readOnly"
            @click="addElement()"
          >
            <svg viewBox="0 0 24 24"><path d="M12 4v16 M4 12h16" /></svg>
          </UiIconButton>
        </nav>

        <div v-show="docked" ref="dockTarget" class="editor-layers-dock" />
        <div class="editor-content">
          <section
            v-if="page === EditorPage.Library || page === EditorPage.Import"
            class="library-workspace interface-properties"
          >
            <UiPageHeader :title="t('modules.interfaces.interfaces')" />
            <nav class="ui-tabs" :aria-label="t('modules.interfaces.interfaces')">
              <button
                type="button"
                :aria-pressed="page === EditorPage.Library"
                @click="page = EditorPage.Library"
              >
                {{ t('modules.interfaces.redesign.library') }}
              </button>
              <button
                type="button"
                :aria-pressed="page === EditorPage.Import"
                :disabled="state.readOnly"
                @click="page = EditorPage.Import"
              >
                {{ t('modules.interfaces.import.title') }}
              </button>
            </nav>
            <div class="interface-sections">
              <InterfaceLibrary
                v-if="page === EditorPage.Library"
                :session="session"
                :unavailable="interfaces.getSnapshot().value?.unavailable ?? []"
                @open="openInterface"
                @create="createInterface"
                @remove-unavailable="void interfaces.removeUnavailable($event)"
              />
              <ImportPage
                v-else
                :interfaces="interfaces"
                :embedded="true"
                @done="page = document === null ? EditorPage.Library : EditorPage.Interface"
              />
            </div>
          </section>
          <NewElementPage
            v-else-if="page === EditorPage.Create"
            :key="state.documentId ?? 'new'"
            :session="session"
            :catalog="interfaces.catalog"
            :parent-id="creationParent"
            @done="finishCreation"
          />
          <template v-else-if="document !== null && snapshot !== null">
            <WidgetInspector
              v-if="selected !== null"
              :key="selected.id"
              v-model:tab="inspectorTab"
              :widget="selected"
              :session="session"
              :catalog="interfaces.catalog"
              :preview="interfaces.preview"
              @open-interface="openInterface(document.id)"
              @add-child="addElement"
            />
            <section v-else class="interface-properties">
              <UiPageHeader
                :title="localization.text(document.name, document.localization, locale)"
              >
                <template #actions>
                  <UiIconButton
                    v-if="interfaceTab === InterfaceTab.Information"
                    :label="t('modules.interfaces.information.enabled')"
                    :active="document.enabled"
                    :aria-pressed="document.enabled"
                    :disabled="state.readOnly"
                    @click="session.toggleDocument()"
                  >
                    <svg viewBox="0 0 20 20">
                      <path d="M2 10s3-5 8-5 8 5 8 5-3 5-8 5-8-5Z" />
                      <circle cx="10" cy="10" r="2" />
                      <path v-if="!document.enabled" d="m3 17 14-14" />
                    </svg>
                  </UiIconButton>
                </template>
              </UiPageHeader>
              <nav class="ui-tabs" :aria-label="t('modules.interfaces.interfaceSettings')">
                <button
                  v-for="value in Object.values(InterfaceTab)"
                  :key="value"
                  type="button"
                  :aria-pressed="interfaceTab === value"
                  @click="interfaceTab = value"
                >
                  {{ t('modules.interfaces.interfaceTabs.' + value) }}
                </button>
              </nav>
              <div class="interface-sections">
                <InterfaceInformation
                  v-if="interfaceTab === InterfaceTab.Information"
                  :document="document"
                  :session="session"
                />
                <FormulasEditor
                  v-else-if="interfaceTab === InterfaceTab.Formulas"
                  :document="document"
                  :session="session"
                />
                <InterfaceTranslations
                  v-else-if="interfaceTab === InterfaceTab.Translations"
                  :document="document"
                  :session="session"
                />
                <ExportPage v-else :document="document" :interfaces="interfaces" />
              </div>
            </section>
          </template>
          <section v-else class="editor-empty">
            <span class="empty-symbol">▦</span>
            <h1>{{ t('modules.interfaces.emptyTitle') }}</h1>
            <p>{{ t('modules.interfaces.emptyHint') }}</p>
            <UiButton :disabled="state.readOnly" @click="addElement()">{{
              t('modules.interfaces.newInterface')
            }}</UiButton>
          </section>
        </div>
      </div>
    </main>
  </FloatingSurface>
  <LayersPanel
    v-if="layersOpen && dockTarget !== null"
    :layout="layersLayout"
    :dock-target="dockTarget"
    :selected-count="state.selectedIds.length"
    @close="closeLayers"
  >
    <aside
      ref="layersRoot"
      class="interfaces-tree"
      :aria-label="t('modules.interfaces.layers')"
      @keydown.esc.stop.prevent="closeLayers"
    >
      <input
        v-model="query"
        class="ui-input"
        type="search"
        :aria-label="t('modules.interfaces.search')"
        :placeholder="t('modules.interfaces.searchElements')"
      />
      <div class="tree-caption">
        {{ t('modules.interfaces.interfaces') }}
        <small>{{ state.workspace.documents.length }}</small>
      </div>
      <div class="tree-scroll">
        <div v-for="entry in documents" :key="entry.id" class="interface-tree-entry">
          <div class="interface-root-row">
            <button
              type="button"
              class="tree-fold"
              :aria-label="t('modules.interfaces.toggleInterface')"
              :aria-expanded="session.layers.isExpanded(entry.id) || query.trim() !== ''"
              @click="session.layers.toggle(entry.id)"
            >
              <svg viewBox="0 0 20 20" aria-hidden="true">
                <path
                  :d="
                    session.layers.isExpanded(entry.id) || query.trim() !== ''
                      ? 'm6 8 4 4 4-4'
                      : 'm8 6 4 4-4 4'
                  "
                />
              </svg>
            </button>
            <button
              type="button"
              class="interface-root"
              :class="{
                'is-selected': state.documentId === entry.id && state.selectedId === null,
              }"
              @click="openInterface(entry.id)"
            >
              <svg class="interface-package-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4 4h6v6H4z M14 4h6v6h-6z M4 14h6v6H4z M14 14h6v6h-6z" /></svg
              ><span>{{ entry.name }}</span
              ><small>{{ session.tree.flatten(entry.roots).length }}</small>
            </button>
            <button
              type="button"
              class="tree-visibility"
              :aria-label="t('modules.interfaces.enabled') + ' · ' + entry.name"
              :aria-pressed="entry.enabled"
              :disabled="state.readOnly"
              @click="session.toggleDocument(entry.id)"
            >
              <svg viewBox="0 0 20 20" aria-hidden="true">
                <template v-if="entry.enabled">
                  <path d="M2 10s3-5 8-5 8 5 8 5-3 5-8 5-8-5-8-5Z" />
                  <circle cx="10" cy="10" r="2" />
                </template>
                <path
                  v-else
                  d="M2 8s3 5 8 5 8-5 8-5 M4 10.5l-1.5 2 M8 12.8l-.5 2.2 M12 12.8l.5 2.2 M16 10.5l1.5 2"
                />
              </svg>
            </button>
          </div>
          <div
            v-if="session.layers.isExpanded(entry.id) || query.trim() !== ''"
            class="interface-widget-tree"
          >
            <WidgetTreeNode
              v-for="widget in entry.roots"
              :key="widget.id"
              :widget="widget"
              :session="session"
              :document-id="entry.id"
              :search="
                entry.name.toLowerCase().includes(query.toLowerCase()) ? '' : query.toLowerCase()
              "
              @select="(id, additive) => openWidget(entry.id, id, additive)"
              @duplicate="
                session.selectDocument(entry.id);
                session.select($event);
                session.duplicateSelected(
                  t('modules.interfaces.copyName', { name: session.selected()?.name ?? '' }),
                );
              "
              @remove="
                session.selectDocument(entry.id);
                session.select($event);
                session.removeSelected();
              "
            />
          </div>
        </div>
        <p v-if="documents.length === 0" class="hint">
          {{ t('modules.interfaces.emptyFilter') }}
        </p>
      </div>
      <UiButton class="is-add" :disabled="state.readOnly" @click="addElement()"
        >＋ {{ t('modules.interfaces.create.elementTitle') }}</UiButton
      >
      <footer class="layers-shortcuts" :aria-label="t('modules.interfaces.hotkeys.title')">
        <span
          ><kbd>Shift</kbd><span>+ {{ t('modules.interfaces.hotkeys.click') }}</span
          ><span>{{ t('modules.interfaces.hotkeys.select') }}</span></span
        >
        <span
          ><kbd>Ctrl</kbd><span>+ {{ t('modules.interfaces.hotkeys.drag') }}</span
          ><span>{{ t('modules.interfaces.hotkeys.group') }}</span></span
        >
        <span><kbd>↑ ↓ ← →</kbd><span>1 px</span><kbd>Shift</kbd><span>10 px</span></span>
      </footer>
    </aside>
  </LayersPanel>
</template>
