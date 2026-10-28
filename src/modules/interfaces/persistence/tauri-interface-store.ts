import type { DesktopCommands } from '../../../core/desktop/desktop-commands';
import type { InterfaceSnapshot } from '../model/interface-snapshot';
import type { InterfaceWorkspace } from '../model/interface-workspace';
import type { InterfaceStore } from './interface-store';

export class TauriInterfaceStore implements InterfaceStore {
  public constructor(private readonly desktop: DesktopCommands) {}

  public async read(): Promise<InterfaceSnapshot> {
    return this.desktop.invoke<InterfaceSnapshot>('interfaces_read');
  }

  public async save(workspace: InterfaceWorkspace): Promise<InterfaceSnapshot> {
    return this.desktop.invoke<InterfaceSnapshot>('interfaces_save', { workspace });
  }

  public async removeUnavailable(folder: string): Promise<InterfaceSnapshot> {
    return this.desktop.invoke<InterfaceSnapshot>('interfaces_remove_unavailable', { folder });
  }
}
