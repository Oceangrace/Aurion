import type { InterfaceSnapshot } from '../model/interface-snapshot';
import type { InterfaceWorkspace } from '../model/interface-workspace';
export interface InterfaceStore {
  read(): Promise<InterfaceSnapshot>;
  save(workspace: InterfaceWorkspace): Promise<InterfaceSnapshot>;
  removeUnavailable(folder: string): Promise<InterfaceSnapshot>;
}
