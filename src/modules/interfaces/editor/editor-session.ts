import type { InterfaceLocalization } from '../model/interface-localization';
import { InterfaceInformationEditor } from './interface-information-editor';
import { GroupScale } from '../scene/group-scale';
import { InterfaceFormat } from '../model/interface-format';
import { shallowReadonly, shallowRef } from 'vue';
import type { SoundPlayer } from '../scene/sound-player';
import type { ShallowRef } from 'vue';
import type { InterfaceDocument } from '../model/interface-document';
import type { InterfaceSnapshot } from '../model/interface-snapshot';
import type { WidgetNode } from '../model/widget-node';
import type { WidgetLayout } from '../model/widget-layout';
import type { Viewport } from '../model/viewport';
import type { InterfaceDisplay } from '../model/interface-display';
import { WidgetKind } from '../model/widget-kind';
import type { InterfaceStore } from '../persistence/interface-store';
import type { EditorState } from './editor-state';
import { SaveStatus } from './save-status';
import { EditorHistory } from './editor-history';
import { WidgetTree } from './widget-tree';
import { WidgetFactory } from './widget-factory';
import { SelectionMovement } from './selection-movement';
import type { EditorInput } from './editor-input';
import type { LayerExpansion } from './layer-expansion';

import type { WidgetAnchors } from '../model/widget-anchors';

export class EditorSession {
  private readonly state = shallowRef<EditorState>({
    workspace: { formatVersion: InterfaceFormat.Current, revision: 0, documents: [] },
    documentId: null,
    selectedId: null,
    selectedIds: [],
    saveStatus: SaveStatus.Clean,
    message: null,
    canUndo: false,
    canRedo: false,
    readOnly: false,
  });

  private readonly history = new EditorHistory();
  private readonly information = new InterfaceInformationEditor();
  private readonly factory = new WidgetFactory();
  public readonly tree = new WidgetTree();
  private gestureStart: readonly InterfaceDocument[] | null = null;
  private gestureMovement: SelectionMovement | null = null;
  private gestureScale = 1;
  private saving = false;
  private viewport: Viewport = { width: 1920, height: 1080 };

  public constructor(
    private readonly store: InterfaceStore,
    private readonly input: EditorInput,
    public readonly sound: SoundPlayer,
    public readonly layers: LayerExpansion,
  ) {}

  public getState(): Readonly<ShallowRef<EditorState>> {
    return shallowReadonly(this.state);
  }

  public isReadOnly(): boolean {
    return this.state.value.readOnly;
  }

  public screen(): Viewport {
    return this.viewport;
  }

  public resize(viewport: Viewport): void {
    this.viewport = viewport;
  }

  public initialize(snapshot: InterfaceSnapshot): void {
    this.layers.load();
    this.viewport = snapshot.viewport;
    this.state.value = {
      ...this.state.value,
      workspace: snapshot.workspace,
      documentId: snapshot.workspace.documents[0]?.id ?? null,
      selectedId: null,
      selectedIds: [],
      message: snapshot.loadError,
      readOnly: snapshot.loadError !== null,
      saveStatus: snapshot.loadError === null ? SaveStatus.Clean : SaveStatus.Error,
    };
  }

  public document(): InterfaceDocument | null {
    return (
      this.state.value.workspace.documents.find(
        (document) => document.id === this.state.value.documentId,
      ) ?? null
    );
  }

  public selected(): WidgetNode | null {
    return this.tree.find(this.document()?.roots ?? [], this.state.value.selectedId);
  }

  public selectDocument(id: string): void {
    this.input.commit();
    this.state.value = { ...this.state.value, documentId: id, selectedId: null, selectedIds: [] };
  }

  public select(id: string | null, additive = false): void {
    this.input.commit();
    const previous = this.state.value.selectedIds;

    const selectedIds = id === null ? [] : [id];

    if (additive === true && id !== null) {
      const next = previous.includes(id)
        ? previous.filter((item) => item !== id)
        : [...previous, id];

      this.state.value = {
        ...this.state.value,
        selectedId: next.at(-1) ?? null,
        selectedIds: next,
      };

      return;
    }

    this.state.value = { ...this.state.value, selectedId: id, selectedIds };
  }

  public selectionIds(): readonly string[] {
    return this.state.value.selectedIds;
  }

  public createDocument(name: string): void {
    const document: InterfaceDocument = {
      id: crypto.randomUUID(),
      name,
      author: '',
      version: '',
      description: '',
      enabled: true,
      roots: [],
      formulas: [],
    };

    this.commit([...this.state.value.workspace.documents, document]);
    this.selectDocument(document.id);
  }

  public applyImported(documents: readonly InterfaceDocument[], id: string): void {
    this.input.commit();
    this.layers.collapseDocument(id);
    this.commit(documents);
    this.selectDocument(id);
  }

  public setFormulas(formulas: InterfaceDocument['formulas']): void {
    this.changeDocument((document) => ({ ...document, formulas }));
  }

  public updateInformation(
    change: Partial<Pick<InterfaceDocument, 'name' | 'author' | 'version' | 'description'>>,
    locale = '',
  ): void {
    this.changeDocument((document) => this.information.update(document, change, locale));
  }

  public setDisplay(display: InterfaceDisplay): void {
    this.changeDocument((document) => ({ ...document, display }));
  }

  public setLocalization(localization: InterfaceLocalization): void {
    this.changeDocument((document) => ({ ...document, localization }));
  }

  public toggleDocument(id = this.state.value.documentId): void {
    this.commit(
      this.state.value.workspace.documents.map((document) =>
        document.id === id ? { ...document, enabled: document.enabled === false } : document,
      ),
    );
  }

  public removeDocument(): void {
    this.input.commit();
    const documents = this.state.value.workspace.documents.filter(
      (document) => document.id !== this.state.value.documentId,
    );

    this.commit(documents);
    this.state.value = {
      ...this.state.value,
      documentId: documents[0]?.id ?? null,
      selectedId: null,
      selectedIds: [],
    };
  }

  public createWidget(created: WidgetNode, parentId: string | null): void {
    if (this.isReadOnly() === true || this.document() === null) {
      return;
    }

    const parent = this.tree.find(this.document()?.roots ?? [], parentId);

    if (parentId !== null && (parent === null || this.tree.isGroup(parent) === false)) {
      return;
    }

    const widget = this.placedWidget(created, parentId);

    this.changeDocument((document) => ({
      ...document,
      roots: this.tree.append(document.roots, parentId, widget),
    }));
    this.select(widget.id);
  }

  private placedWidget(created: WidgetNode, parentId: string | null): WidgetNode {
    const roots = this.document()?.roots ?? [];

    const siblings = parentId === null ? roots : (this.tree.find(roots, parentId)?.children ?? []);

    const offset = siblings.length * 16;

    return {
      ...created,
      layout: {
        ...created.layout,
        x: (parentId === null ? this.viewport.width / 2 - created.layout.width / 2 : 0) + offset,
        y: (parentId === null ? this.viewport.height / 2 - created.layout.height / 2 : 0) + offset,
      },
    };
  }

  public resetPositions(): void {
    this.changeDocument((document) => ({
      ...document,
      roots: document.roots.map((node, index) => ({
        ...node,
        layout: {
          ...node.layout,
          x: this.viewport.width / 2 - new GroupScale().frame(node).width / 2 + index * 16,
          y: this.viewport.height / 2 - new GroupScale().frame(node).height / 2 + index * 16,
        },
      })),
    }));
  }

  public nudge(x: number, y: number): void {
    const document = this.document();

    if (document !== null && this.selectionIds().length > 1 === true) {
      const movement = new SelectionMovement(
        document,
        this.selectionIds(),
        this.tree,
        this.viewport,
      );

      this.changeDocument((current) => ({ ...current, roots: movement.move(x, y) }));

      return;
    }

    this.nudgeOne(x, y);
  }

  private nudgeOne(x: number, y: number): void {
    const selected = this.selected();

    if (selected === null || selected.group.locked === true) {
      return;
    }

    const roots = this.document()?.roots ?? [];

    const world = this.tree.world(roots, selected.id);

    const parent = this.tree.parent(roots, selected.id);

    if (world === null || parent?.kind === WidgetKind.DynamicGroup) {
      return;
    }

    const factor = this.tree.inheritedScale(roots, selected.id);

    const deltaX = Math.max(-world.x, Math.min(x, this.viewport.width - world.x - world.width));

    const deltaY = Math.max(-world.y, Math.min(y, this.viewport.height - world.y - world.height));

    this.updateSelected((node) => ({
      ...node,
      layout: {
        ...node.layout,
        x: node.layout.x + deltaX / factor,
        y: node.layout.y + deltaY / factor,
      },
    }));
  }

  public coordinate(axis: 'x' | 'y'): number {
    const selected = this.selected();

    if (selected === null) {
      return 0;
    }

    return selected.layout[axis] - this.coordinateOrigin(selected, axis);
  }

  public updateCoordinate(axis: 'x' | 'y', value: number): void {
    const selected = this.selected();

    if (selected === null) {
      return;
    }

    const origin = this.coordinateOrigin(selected, axis);

    this.updateSelected((node) => ({
      ...node,
      layout: { ...node.layout, [axis]: value + origin },
    }));
  }

  private coordinateOrigin(widget: WidgetNode, axis: 'x' | 'y'): number {
    if (this.tree.parent(this.document()?.roots ?? [], widget.id) !== null) {
      return 0;
    }

    return (axis === 'x' ? this.viewport.width : this.viewport.height) / 2;
  }

  public updateSelected(change: (widget: WidgetNode) => WidgetNode): void {
    const id = this.state.value.selectedId;

    if (id === null) {
      return;
    }

    this.changeDocument((document) => ({
      ...document,
      roots: this.tree.update(document.roots, id, change),
    }));
  }

  public renameWidget(name: string): void {
    this.updateSelected((widget) => ({ ...widget, name }));
  }

  public toggleWidget(id: string): void {
    this.changeDocument((document) => ({
      ...document,
      roots: this.tree.update(document.roots, id, (widget) => ({
        ...widget,
        enabled: widget.enabled === false,
      })),
    }));
  }

  public removeSelected(): void {
    this.input.commit();
    const id = this.state.value.selectedId;

    if (id === null) {
      return;
    }

    this.changeDocument((document) => ({
      ...document,
      roots: this.tree.remove(document.roots, id),
    }));
    this.select(null);
  }

  public duplicateSelected(copyName: string): void {
    this.input.commit();
    const selected = this.selected();

    const document = this.document();

    if (selected === null || document === null) {
      return;
    }

    const copy = this.factory.duplicate(selected, copyName);

    const parent = this.tree.parent(document.roots, selected.id);

    this.changeDocument((current) => ({
      ...current,
      roots: this.tree.append(current.roots, parent?.id ?? null, copy),
    }));
    this.select(copy.id);
  }

  public reorder(direction: number): void {
    this.input.commit();
    const id = this.state.value.selectedId;

    if (id === null) {
      return;
    }

    this.changeDocument((document) => ({
      ...document,
      roots: this.tree.reorder(document.roots, id, direction),
    }));
  }

  public reparent(parentId: string | null): void {
    this.input.commit();
    const widget = this.selected();

    const document = this.document();

    if (widget === null || document === null) {
      return;
    }

    if (this.tree.canReparent(document.roots, widget, parentId) === false) {
      return;
    }

    const moved = this.tree.relocated(document.roots, widget, parentId);

    this.changeDocument((current) => ({
      ...current,
      roots: this.tree.append(this.tree.remove(current.roots, widget.id), parentId, moved),
    }));
  }

  public selectWidget(id: string): void {
    const document = this.state.value.workspace.documents.find(
      (entry) => this.tree.find(entry.roots, id) !== null,
    );

    if (document === undefined) {
      return;
    }

    if (document.id !== this.state.value.documentId) {
      this.selectDocument(document.id);
    }

    this.select(id);
  }

  public beginGesture(id: string, moving = false, scale = 1): void {
    this.input.commit();

    if (moving === false || this.selectionIds().includes(id) === false) {
      this.selectWidget(id);
    }

    this.gestureStart = this.state.value.workspace.documents;
    const document = this.document();

    this.gestureScale = scale;
    this.gestureMovement =
      moving && this.selectionIds().length > 1 && document !== null
        ? new SelectionMovement(document, this.selectionIds(), this.tree, this.viewport)
        : null;
  }

  public updateLayout(id: string, layout: WidgetLayout, anchors: WidgetAnchors): void {
    if (this.gestureMovement !== null) {
      const roots = this.gestureMovement.drag(id, layout, this.gestureScale);

      this.replaceDocuments(
        this.state.value.workspace.documents.map((document) =>
          document.id === this.state.value.documentId ? { ...document, roots } : document,
        ),
      );

      return;
    }

    const previous = this.tree.find(this.document()?.roots ?? [], id);

    if (
      previous === null ||
      (this.tree.sameLayout(previous.layout, layout) === true &&
        JSON.stringify(previous.anchors) === JSON.stringify(anchors))
    ) {
      return;
    }

    const documents = this.state.value.workspace.documents.map((document) => ({
      ...document,
      roots: this.tree.update(document.roots, id, (widget) => ({ ...widget, layout, anchors })),
    }));

    this.replaceDocuments(documents);
  }

  public endGesture(cancelled = false): void {
    const before = this.gestureStart;

    this.gestureStart = null;
    this.gestureMovement = null;

    if (before === null || before === this.state.value.workspace.documents) {
      return;
    }

    if (cancelled === true) {
      this.replaceDocuments(before);

      return;
    }

    this.history.record(before);
    this.syncHistory();
  }

  public undo(): void {
    this.input.commit();
    this.endGesture();
    const previous = this.history.undo(this.state.value.workspace.documents);

    if (previous !== null) {
      this.replaceDocuments(previous);
    }

    this.syncHistory();
  }

  public redo(): void {
    this.input.commit();
    const next = this.history.redo(this.state.value.workspace.documents);

    if (next !== null) {
      this.replaceDocuments(next);
    }

    this.syncHistory();
  }

  public async save(): Promise<boolean> {
    this.input.commit();

    if (this.saving === true) {
      return false;
    }

    if (this.state.value.readOnly === true) {
      return false;
    }

    if (this.state.value.saveStatus === SaveStatus.Clean) {
      return true;
    }

    this.saving = true;
    const workspace = this.state.value.workspace;

    this.state.value = { ...this.state.value, saveStatus: SaveStatus.Saving, message: null };

    try {
      const result = await this.store.save(workspace);

      const unchanged = workspace.documents === this.state.value.workspace.documents;

      this.state.value = {
        ...this.state.value,
        workspace: { ...this.state.value.workspace, revision: result.workspace.revision },
        saveStatus: unchanged ? SaveStatus.Clean : SaveStatus.Dirty,
      };

      return unchanged;
    } catch (error) {
      this.report(error);

      return false;
    } finally {
      this.saving = false;
    }
  }

  public report(error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);

    this.state.value = { ...this.state.value, message, saveStatus: SaveStatus.Error };
  }

  private changeDocument(change: (document: InterfaceDocument) => InterfaceDocument): void {
    const id = this.state.value.documentId;

    this.commit(
      this.state.value.workspace.documents.map((document) =>
        document.id === id ? change(document) : document,
      ),
    );
  }

  private commit(documents: readonly InterfaceDocument[]): void {
    if (this.state.value.readOnly === true) {
      return;
    }

    this.history.record(this.state.value.workspace.documents);
    this.replaceDocuments(documents);
    this.syncHistory();
  }

  private replaceDocuments(documents: readonly InterfaceDocument[]): void {
    const documentId = documents.some((document) => document.id === this.state.value.documentId)
      ? this.state.value.documentId
      : (documents[0]?.id ?? null);

    this.state.value = {
      ...this.state.value,
      documentId,
      workspace: { ...this.state.value.workspace, documents },
      saveStatus: SaveStatus.Dirty,
      message: null,
    };

    const selectedIds = this.selectionIds().filter(
      (id) => this.tree.find(this.document()?.roots ?? [], id) !== null,
    );

    this.state.value = {
      ...this.state.value,
      selectedIds,
      selectedId:
        this.selected() === null ? (selectedIds.at(-1) ?? null) : this.state.value.selectedId,
    };
  }

  private syncHistory(): void {
    this.state.value = {
      ...this.state.value,
      canUndo: this.history.canUndo(),
      canRedo: this.history.canRedo(),
    };
  }
}
