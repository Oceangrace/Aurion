import { Direction } from '../model/direction';
import { defaultLoadRules } from '../model/load-rules';
import { defaultWidgetSounds } from '../model/widget-sounds';
import { defaultWidgetStyle } from '../model/widget-style';
import { defaultGroupSettings } from '../model/group-settings';
import type { WidgetNode } from '../model/widget-node';
import type { WidgetLayout } from '../model/widget-layout';
import { WidgetKind } from '../model/widget-kind';

import { defaultWidgetAnchors } from '../model/widget-anchors';

export class WidgetFactory {
  public create(kind: WidgetKind, name: string): WidgetNode {
    const group = kind === WidgetKind.Group || kind === WidgetKind.DynamicGroup;

    return {
      ruleSelections: {},
      imageSelection: null,
      rules: null,
      binding: null,
      headerButtons: [],
      entityId: null,
      load: { ...defaultLoadRules },
      sounds: { ...defaultWidgetSounds },
      id: crypto.randomUUID(),
      name,
      kind,
      enabled: true,
      layout: this.layout(kind),
      anchors: { ...defaultWidgetAnchors },
      appearance: {
        text: kind === WidgetKind.Text ? name : '',
        color: '#e8e8e8',
        background: '#141414cc',
        borderColor: group ? '#00000000' : '#d6b26e',
        fontSize: 20,
        opacity: 1,
        iconPath: null,
        customImage: null,
      },
      style: {
        ...defaultWidgetStyle,
        borderWidth: group ? 0 : 1,
        barDirection: kind === WidgetKind.Bar ? Direction.Right : null,
      },
      group: { ...defaultGroupSettings, fillEnabled: false },
      conditions: [],
      trigger: null,
      children: [],
    };
  }

  private layout(kind: WidgetKind): WidgetLayout {
    switch (kind) {
      case WidgetKind.DynamicGroup:
      case WidgetKind.Group:
        return { x: 0, y: 0, width: 320, height: 160 };
      case WidgetKind.Text:
        return { x: 0, y: 0, width: 240, height: 48 };
      case WidgetKind.Bar:
        return { x: 0, y: 0, width: 240, height: 28 };
      case WidgetKind.Icon:
        return { x: 0, y: 0, width: 48, height: 48 };
    }
  }

  public duplicate(widget: WidgetNode, copyName: string): WidgetNode {
    return {
      ...widget,
      id: crypto.randomUUID(),
      name: copyName,
      layout: { ...widget.layout, x: widget.layout.x + 16, y: widget.layout.y + 16 },
      children: widget.children.map((child) => this.copyChild(child)),
    };
  }

  private copyChild(widget: WidgetNode): WidgetNode {
    return {
      ...widget,
      id: crypto.randomUUID(),
      children: widget.children.map((child) => this.copyChild(child)),
    };
  }
}
