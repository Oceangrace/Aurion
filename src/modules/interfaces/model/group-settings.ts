import { Direction } from './direction';
import { GroupGrow } from './group-grow';
import { GroupSort } from './group-sort';
import { TextAlignment } from './text-alignment';
import { GroupFlow } from './group-flow';

export interface GroupSettings {
  readonly fillEnabled: boolean;
  readonly headerEnabled: boolean;
  readonly headerHeight: number;
  readonly headerText: string;
  readonly headerColor: string;
  readonly headerBackground: string;
  readonly scale: number;
  readonly padding: number;
  readonly locked: boolean;
  readonly flow: GroupFlow;
  readonly gap: number;
  readonly columns: number;
  readonly headerFontSize: number;
  readonly headerAlignment: TextAlignment;
  readonly sort: GroupSort;
  readonly grow: GroupGrow;
  readonly limit: number;
  readonly motionMs: number;
  readonly travel: Direction;
  readonly spanSeconds: number;
  readonly fillOpacity: number | null;
  readonly headerOpacity: number | null;
}

export const defaultGroupSettings: GroupSettings = {
  fillEnabled: true,
  headerEnabled: false,
  headerHeight: 28,
  headerText: '',
  headerColor: '#d6b26e',
  headerBackground: '#141414',
  scale: 100,
  padding: 0,
  locked: false,
  flow: GroupFlow.Row,
  gap: 8,
  columns: 4,
  headerFontSize: 14,
  headerAlignment: TextAlignment.Left,
  sort: GroupSort.Manual,
  grow: GroupGrow.Forward,
  limit: 0,
  motionMs: 0,
  travel: Direction.Left,
  spanSeconds: 30,
  fillOpacity: null,
  headerOpacity: null,
};
