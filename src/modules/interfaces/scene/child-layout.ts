import { Direction } from '../model/direction';
import { GroupScale } from './group-scale';
import type { PreviewVisual } from '../model/preview-visual';
import type { WidgetNode } from '../model/widget-node';
import type { WidgetLayout } from '../model/widget-layout';
import { WidgetKind } from '../model/widget-kind';
import { GroupSort } from '../model/group-sort';
import { GroupGrowOffset } from './group-grow-offset';
import { GroupFlow } from '../model/group-flow';
import { GroupDataSource } from '../model/rules/group-data-source';
import { WidgetAnchorGeometry } from './widget-anchor-geometry';
export class ChildLayout {
  private readonly scale = new GroupScale();
  private readonly grow = new GroupGrowOffset();

  public content(parent: WidgetNode, layout: WidgetLayout): WidgetLayout {
    const padding = parent.group.padding;

    const header = parent.group.headerEnabled ? parent.group.headerHeight : 0;

    return {
      x: padding,
      y: padding + header,
      width: Math.max(8, layout.width - padding * 2),
      height: Math.max(8, layout.height - padding * 2 - header),
    };
  }

  public resolve(
    node: WidgetNode,
    parent: WidgetNode,
    layout: WidgetLayout,
    includeDisabled = false,
    visuals: Readonly<Record<string, PreviewVisual>> = {},
  ): WidgetLayout {
    const content = this.content(parent, layout);

    if (
      parent.kind !== WidgetKind.DynamicGroup ||
      (parent.binding !== null && parent.binding.source !== GroupDataSource.SkillHistory)
    ) {
      const factor = this.scale.factor(node);

      const child = new WidgetAnchorGeometry().resolve(
        this.scale.frame(node),
        node.anchors,
        content,
      );

      return {
        x: content.x + child.x,
        y: content.y + child.y,
        width: child.width / factor,
        height: child.height / factor,
      };
    }

    if (parent.group.flow === GroupFlow.Timeline) {
      return this.timeline(node, parent, content, visuals, includeDisabled);
    }

    const siblings = this.siblings(node, parent, includeDisabled, visuals);

    const index = Math.max(
      0,
      siblings.findIndex((child) => child.id === node.id),
    );

    const offset = this.offset(parent, siblings, index, content);

    return { ...node.layout, x: content.x + offset.x, y: content.y + offset.y };
  }

  public siblings(
    node: WidgetNode,
    parent: WidgetNode,
    includeDisabled: boolean,
    visuals: Readonly<Record<string, PreviewVisual>>,
  ): readonly WidgetNode[] {
    let siblings = parent.children.filter(
      (child) =>
        (includeDisabled || child.enabled || child.id === node.id) &&
        (includeDisabled || visuals[child.id]?.visible !== false),
    );

    if (parent.group.sort !== GroupSort.Manual) {
      siblings = [...siblings].sort((a, b) => this.compare(a, b, parent.group.sort, visuals));
    }

    if (parent.group.limit > 0 === true) {
      siblings = siblings.slice(0, parent.group.limit);
    }

    return siblings;
  }

  public visible(
    node: WidgetNode,
    parent: WidgetNode,
    editable: boolean,
    visuals: Readonly<Record<string, PreviewVisual>>,
  ): boolean {
    if (parent.kind !== WidgetKind.DynamicGroup || parent.binding !== null) {
      return true;
    }

    if (
      parent.group.flow === GroupFlow.Timeline &&
      editable === false &&
      Number.isFinite(this.remaining(node, visuals)) === false
    ) {
      return false;
    }

    if (parent.group.limit === 0) {
      return true;
    }

    return this.siblings(node, parent, editable, visuals).some((child) => child.id === node.id);
  }

  private compare(
    a: WidgetNode,
    b: WidgetNode,
    sort: GroupSort,
    visuals: Readonly<Record<string, PreviewVisual>>,
  ): number {
    const left = this.remaining(a, visuals);

    const right = this.remaining(b, visuals);

    if (Number.isFinite(left) === false) {
      return Number.isFinite(right) ? 1 : 0;
    }

    if (Number.isFinite(right) === false) {
      return -1;
    }

    return (left - right) * (sort === GroupSort.RemainingDesc ? -1 : 1);
  }

  private offset(
    parent: WidgetNode,
    siblings: readonly WidgetNode[],
    index: number,
    content: WidgetLayout,
  ): { x: number; y: number } {
    const gap = parent.group.gap;

    const before = siblings.slice(0, index);

    if (parent.group.flow === GroupFlow.Row) {
      const total =
        siblings.reduce((sum, child) => sum + this.scale.frame(child).width, 0) +
        Math.max(0, siblings.length - 1) * gap;

      return {
        x:
          this.grow.resolve(parent.group.grow, true, content.width, total) +
          before.reduce((sum, child) => sum + this.scale.frame(child).width + gap, 0),
        y: 0,
      };
    }

    if (parent.group.flow === GroupFlow.Column) {
      const total =
        siblings.reduce((sum, child) => sum + this.scale.frame(child).height, 0) +
        Math.max(0, siblings.length - 1) * gap;

      return {
        x: 0,
        y:
          this.grow.resolve(parent.group.grow, false, content.height, total) +
          before.reduce((sum, child) => sum + this.scale.frame(child).height + gap, 0),
      };
    }

    const width = Math.max(8, ...siblings.map((child) => this.scale.frame(child).width));

    const height = Math.max(8, ...siblings.map((child) => this.scale.frame(child).height));

    const columns = Math.min(parent.group.columns, siblings.length);

    const rows = Math.ceil(siblings.length / parent.group.columns);

    return {
      x:
        this.grow.resolve(parent.group.grow, true, content.width, columns * (width + gap) - gap) +
        (index % parent.group.columns) * (width + gap),
      y:
        this.grow.resolve(parent.group.grow, false, content.height, rows * (height + gap) - gap) +
        Math.floor(index / parent.group.columns) * (height + gap),
    };
  }

  private timeline(
    node: WidgetNode,
    parent: WidgetNode,
    content: WidgetLayout,
    visuals: Readonly<Record<string, PreviewVisual>>,
    editing: boolean,
  ): WidgetLayout {
    const remaining = this.remaining(node, visuals);

    const sample = this.sample(node, parent, editing);

    const fraction = Number.isFinite(remaining)
      ? Math.min(1, Math.max(0, remaining / parent.group.spanSeconds))
      : sample;

    const vertical = parent.group.travel === Direction.Up || parent.group.travel === Direction.Down;

    const reverse = parent.group.travel === Direction.Left || parent.group.travel === Direction.Up;

    const available = vertical
      ? content.height - this.scale.frame(node).height
      : content.width - this.scale.frame(node).width;

    const offset = (reverse ? fraction : 1 - fraction) * available;

    return {
      ...node.layout,
      x:
        content.x +
        (vertical ? Math.max(0, content.width - this.scale.frame(node).width) / 2 : offset),
      y:
        content.y +
        (vertical ? offset : Math.max(0, content.height - this.scale.frame(node).height) / 2),
    };
  }

  private sample(node: WidgetNode, parent: WidgetNode, editing: boolean): number {
    const index = Math.max(
      0,
      parent.children.findIndex((child) => child.id === node.id),
    );

    return editing ? 0.15 + (index / Math.max(1, parent.children.length)) * 0.7 : 0;
  }

  private remaining(node: WidgetNode, visuals: Readonly<Record<string, PreviewVisual>>): number {
    return visuals[node.id]?.remaining ?? Number.POSITIVE_INFINITY;
  }
}
