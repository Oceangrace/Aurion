import type { InterfaceWorkspace } from './interface-workspace';
import type { UnavailableProfile } from './unavailable-profile';
import type { Viewport } from './viewport';

export interface InterfaceSnapshot {
  readonly workspace: InterfaceWorkspace;
  readonly viewport: Viewport;
  readonly loadError: string | null;
  readonly unavailable: readonly UnavailableProfile[];
}
