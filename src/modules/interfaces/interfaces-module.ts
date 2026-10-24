import { shallowReadonly, shallowRef, watch, computed } from 'vue';
import { LiveAppearance } from './scene/live-appearance';
import type { PreviewVisual } from './model/preview-visual';
import type { ComputedRef } from 'vue';
import { SoundPlayer } from './scene/sound-player';
import { AppearancePreview } from './editor/appearance-preview';
import type { ShallowRef } from 'vue';
import type { DesktopClient } from '../../core/desktop/desktop-client';
import type { InterfaceStore } from './persistence/interface-store';
import type { InterfaceSnapshot } from './model/interface-snapshot';
import type { InterfaceEvents } from './model/interface-events';
import type { CatalogClient } from './triggers/catalog-client';
import { EditorSession } from './editor/editor-session';
import { EditorInput } from './editor/editor-input';
import { LayerExpansion } from './editor/layer-expansion';
import { EditorAction } from './model/editor-action';
import type { InterfaceDocument } from './model/interface-document';
import type { WidgetNode } from './model/widget-node';
import type { ImportPreview } from './model/import-preview';
import type { InterfaceWorkspace } from './model/interface-workspace';
import type { SoundPlayback } from './scene/sound-playback';
import { ConditionProperty } from './model/condition-property';
import { GroupDataSource } from './model/rules/group-data-source';
import { SoundCue } from './model/sound-cue';
import { ExitRequest } from './model/exit-request';
import { SaveStatus } from './editor/save-status';

export class InterfacesModule {
  public readonly session: EditorSession;
  public readonly preview: AppearancePreview;
  public readonly live: LiveAppearance;
  private readonly visuals: ComputedRef<Readonly<Record<string, PreviewVisual>>>;
  private readonly input = new EditorInput();
  private readonly speechCues = new Set<string>([SoundCue.Speech]);
  private readonly snapshot = shallowRef<InterfaceSnapshot | null>(null);
  private readonly ready = shallowRef(false);
  private readonly editing = shallowRef(false);
  private readonly blockedExit = shallowRef<ExitRequest | null>(null);
  private readonly hudEnabled = shallowRef(true);
  private readonly gamePanel = shallowRef(false);
  private gamePanelRevision = 0;
  private readonly foreground = shallowRef(false);
  private foregroundRevision = 0;
  private readonly gameAttached = shallowRef<boolean | null>(null);
  private attachmentRevision = 0;
  private readonly subscriptions: (() => void)[] = [];

  public constructor(
    private readonly desktop: DesktopClient<InterfaceEvents>,
    private readonly store: InterfaceStore,
    public readonly catalog: CatalogClient,
    private readonly locale: () => string,
  ) {
    this.session = new EditorSession(
      store,
      this.input,
      new SoundPlayer(desktop),
      new LayerExpansion(window.localStorage, (error) => {
        this.session.report(error);
      }),
    );
    this.preview = new AppearancePreview(desktop, locale);
    this.live = new LiveAppearance(desktop, locale);
    this.visuals = computed(() => {
      if (this.editing.value === true || this.preview.enabled.value === true) {
        return this.editorVisuals();
      }

      return this.live.getFrame().value?.visuals ?? {};
    });
  }

  private editorVisuals(): Readonly<Record<string, PreviewVisual>> {
    if (this.preview.enabled.value === true) {
      return this.preview.getVisuals().value;
    }

    const visuals = { ...this.preview.getVisuals().value };

    const nodes = this.session.tree.flatten(
      this.session.getState().value.workspace.documents.flatMap((document) => document.roots),
    );

    for (const node of nodes.filter((node) => node.binding?.source === GroupDataSource.Effects)) {
      const observed = this.live.getFrame().value?.visuals[node.id];

      if (observed !== undefined) {
        visuals[node.id] = {
          ...observed,
          ...visuals[node.id],
          instances: observed.instances,
          simulating: false,
        };
      }
    }

    return visuals;
  }

  public getVisuals(): Readonly<ComputedRef<Readonly<Record<string, PreviewVisual>>>> {
    return this.visuals;
  }

  public getSnapshot(): Readonly<ShallowRef<InterfaceSnapshot | null>> {
    return shallowReadonly(this.snapshot);
  }

  public getReady(): Readonly<ShallowRef<boolean>> {
    return shallowReadonly(this.ready);
  }

  public getEditing(): Readonly<ShallowRef<boolean>> {
    return shallowReadonly(this.editing);
  }

  public getBlockedExit(): Readonly<ShallowRef<ExitRequest | null>> {
    return shallowReadonly(this.blockedExit);
  }

  public getHudEnabled(): Readonly<ShallowRef<boolean>> {
    return shallowReadonly(this.hudEnabled);
  }

  public surfaceVisible(): boolean {
    return this.hudEnabled.value === true && this.gameSurfaceVisible();
  }

  public gameSurfaceVisible(): boolean {
    return (
      this.editing.value === true ||
      this.preview.enabled.value === true ||
      (this.live.getFrame().value?.inWorld === true &&
        this.foreground.value === true &&
        this.gamePanel.value === false)
    );
  }

  public toggleHud(): void {
    this.hudEnabled.value = this.hudEnabled.value === false;
  }

  public getForeground(): Readonly<ShallowRef<boolean>> {
    return shallowReadonly(this.foreground);
  }

  public getGameAttached(): Readonly<ShallowRef<boolean | null>> {
    return shallowReadonly(this.gameAttached);
  }

  public open(): void {
    this.editing.value = true;
  }

  public async start(): Promise<void> {
    try {
      this.subscriptions.push(
        await this.desktop.listen('interfaces:changed', (snapshot) => {
          this.snapshot.value = snapshot;
          this.resize();
        }),
        await this.desktop.listen('interfaces:open-requested', () => {
          this.open();
        }),
        await this.desktop.listen('overlay:game-panel', (open) => {
          this.gamePanelRevision += 1;
          this.gamePanel.value = open;
        }),
        await this.desktop.listen('overlay:foreground', (active) => {
          this.foregroundRevision += 1;
          this.foreground.value = active;
          this.live.setForeground(active);
        }),
        await this.desktop.listen('overlay:attached', (attached) => {
          this.attachmentRevision += 1;
          this.gameAttached.value = attached;
        }),
      );

      const panelRevision = this.gamePanelRevision;

      const foregroundRevision = this.foregroundRevision;

      const attachmentRevision = this.attachmentRevision;

      const [panel, foreground, attached] = await Promise.all([
        this.desktop.invoke<boolean>('overlay_game_panel'),
        this.desktop.invoke<boolean>('overlay_foreground'),
        this.desktop.invoke<boolean>('overlay_game_attached'),
      ]);

      if (this.gamePanelRevision === panelRevision) {
        this.gamePanel.value = panel;
      }

      if (this.foregroundRevision === foregroundRevision) {
        this.foreground.value = foreground;
      }

      if (this.attachmentRevision === attachmentRevision) {
        this.gameAttached.value = attached;
      }

      const initial = await this.store.read();

      this.snapshot.value = initial;
      this.session.initialize(initial);
      this.resize();
      window.addEventListener('resize', this.resize);
      this.subscriptions.push(() => {
        window.removeEventListener('resize', this.resize);
      });
      this.preview.sync(initial.workspace);
      this.live.setForeground(this.foreground.value);
      this.live.start();
      this.prepareSpeech(initial.workspace);
      this.subscriptions.push(
        watch(this.locale, () => {
          this.preview.sync(this.session.getState().value.workspace);
          this.prepareSpeech(this.session.getState().value.workspace);
        }),
        watch(this.getVisuals(), (visuals) => {
          this.session.tree.setVisuals(visuals);
        }),
      );
      this.subscriptions.push(
        watch(
          () => this.session.getState().value.workspace,
          (workspace) => {
            this.preview.sync(workspace);
            this.prepareSpeech(workspace);
          },
        ),
      );
    } catch (error) {
      this.session.report(error);
    }

    this.ready.value = true;
  }

  public saveExport(text: string): Promise<string | null> {
    return this.desktop.invoke<string | null>('interfaces_export_file', { text });
  }

  private readonly resize = (): void => {
    const snapshot = this.snapshot.value;

    if (snapshot === null) {
      return;
    }

    const viewport = { width: window.innerWidth, height: window.innerHeight };

    this.session.resize(viewport);
    this.snapshot.value = { ...snapshot, viewport };
  };

  public saveProfile(document: InterfaceDocument): Promise<string | null> {
    return this.desktop.invoke<string | null>('interfaces_export_folder', { document });
  }

  public inspectProfile(): Promise<ImportPreview | null> {
    return this.desktop.invoke<ImportPreview | null>('interfaces_import_folder');
  }

  private prepareSpeech(workspace: InterfaceWorkspace): void {
    const widgets = workspace.documents.flatMap((document) =>
      this.session.tree.flatten(document.roots),
    );

    this.session.sound.prepare(widgets.flatMap((widget) => this.speechRequests(widget)));
  }

  private speechRequests(widget: WidgetNode): SoundPlayback[] {
    const common = {
      volume: widget.sounds.volume,
      voice: widget.sounds.voice,
      rate: widget.sounds.voiceRate,
      owner: widget.id,
    };

    const events = [
      { ...common, cue: widget.sounds.show, text: widget.sounds.showText },
      { ...common, cue: widget.sounds.hide, text: widget.sounds.hideText },
    ];

    const conditions = widget.conditions.flatMap((condition) =>
      condition.changes.flatMap((change) => {
        if (
          change.property !== ConditionProperty.Sound ||
          this.speechCues.has(change.value) === false
        ) {
          return [];
        }

        return [{ ...common, cue: SoundCue.Speech, text: change.speechText ?? '' }];
      }),
    );

    return [...events, ...conditions];
  }

  public inspectImport(text: string): Promise<ImportPreview> {
    return this.desktop.invoke('interfaces_import', { text });
  }

  public export(document: InterfaceDocument): Promise<string> {
    return this.desktop.invoke('interfaces_export', {
      document,
    });
  }

  public async importDocument(
    document: InterfaceDocument,
    copy: boolean,
    copyName: string,
  ): Promise<void> {
    if (this.session.isReadOnly() === true) {
      return;
    }

    const clone = (node: WidgetNode): WidgetNode => ({
      ...node,
      id: crypto.randomUUID(),
      children: node.children.map(clone),
    });

    const source = { ...document, imported: true };

    const imported = copy
      ? {
          ...source,
          id: crypto.randomUUID(),
          name: copyName,
          roots: document.roots.map(clone),
        }
      : source;

    const workspace = this.session.getState().value.workspace;

    const documents = workspace.documents.some((entry) => entry.id === imported.id)
      ? workspace.documents.map((entry) => (entry.id === imported.id ? imported : entry))
      : [...workspace.documents, imported];

    await this.desktop.invoke('interfaces_validate', { workspace: { ...workspace, documents } });

    if (this.session.getState().value.workspace !== workspace) {
      throw new Error('The editor changed during import. Import again.');
    }

    this.session.applyImported(documents, imported.id);
  }

  public execute(action: EditorAction): void {
    const actions = new Map<EditorAction, () => void>([
      [
        EditorAction.Save,
        () => {
          void this.session.save();
        },
      ],
      [
        EditorAction.Undo,
        () => {
          this.session.undo();
        },
      ],
      [
        EditorAction.Redo,
        () => {
          this.session.redo();
        },
      ],
      [
        EditorAction.Remove,
        () => {
          this.session.removeSelected();
        },
      ],
      [
        EditorAction.Deselect,
        () => {
          this.session.select(null);
        },
      ],
      [
        EditorAction.Duplicate,
        () => {
          this.session.duplicateSelected(this.session.selected()?.name ?? '');
        },
      ],
      [
        EditorAction.Left,
        () => {
          this.session.nudge(-1, 0);
        },
      ],
      [
        EditorAction.Right,
        () => {
          this.session.nudge(1, 0);
        },
      ],
      [
        EditorAction.Up,
        () => {
          this.session.nudge(0, -1);
        },
      ],
      [
        EditorAction.Down,
        () => {
          this.session.nudge(0, 1);
        },
      ],
      [
        EditorAction.LeftFast,
        () => {
          this.session.nudge(-10, 0);
        },
      ],
      [
        EditorAction.RightFast,
        () => {
          this.session.nudge(10, 0);
        },
      ],
      [
        EditorAction.UpFast,
        () => {
          this.session.nudge(0, -10);
        },
      ],
      [
        EditorAction.DownFast,
        () => {
          this.session.nudge(0, 10);
        },
      ],
    ]);

    actions.get(action)?.();
  }

  public async close(): Promise<void> {
    if ((await this.saved(ExitRequest.Close)) === true) {
      this.finishClose();
    }
  }

  public async quit(): Promise<void> {
    if ((await this.saved(ExitRequest.Quit)) === true) {
      await this.exit();
    }
  }

  /** Leaves after a failed save: closing keeps the edits in the editor, quitting drops them. */
  public async continueExit(): Promise<void> {
    const request = this.blockedExit.value;

    this.blockedExit.value = null;

    if (request === ExitRequest.Close) {
      this.finishClose();
    }

    if (request === ExitRequest.Quit) {
      await this.exit();
    }
  }

  public async removeUnavailable(folder: string): Promise<void> {
    try {
      this.snapshot.value = await this.store.removeUnavailable(folder);
    } catch (error) {
      this.session.report(error);
    }
  }

  public cancelExit(): void {
    this.blockedExit.value = null;
  }

  private async saved(request: ExitRequest): Promise<boolean> {
    if (this.session.isReadOnly() === true || (await this.session.save()) === true) {
      return true;
    }

    // A save already in progress or edits made during it are not failures; only an error asks.
    if (this.session.getState().value.saveStatus === SaveStatus.Error) {
      this.blockedExit.value = request;
    }

    return false;
  }

  private finishClose(): void {
    this.preview.enable(false);
    this.editing.value = false;
  }

  private async exit(): Promise<void> {
    try {
      await this.desktop.invoke('app_exit');
    } catch (error) {
      this.session.report(error);
    }
  }

  public dispose(): void {
    this.preview.dispose();
    this.live.dispose();
    this.session.sound.dispose();

    for (const unsubscribe of this.subscriptions) {
      unsubscribe();
    }
  }
}
