import { ImageSource } from './image-source';
import type { Direction } from './direction';
import { AnimationChange } from './animation-change';
import { ClassColorSource } from './class-color-source';
import { ClassColorTarget } from './class-color-target';
import { BarTexture } from './bar-texture';
import type { HealingPredictionTarget } from './healing-prediction-target';
import { AnimationWhen } from './animation-when';
import { BarIconAnchor } from './bar-icon-anchor';
import { BorderStyle } from './border-style';
import { GlowMode } from './glow-mode';
import { MotionKind } from './motion-kind';
import { ProgressSource } from './progress-source';
import { TextPosition } from './text-position';
import { FontFamily } from './font-family';
import { ImageFit } from './image-fit';
import { TextAlignment } from './text-alignment';
import { TextAnchor } from './text-anchor';
import { WidgetAnimation } from './widget-animation';

export interface WidgetStyle {
  readonly classColorSource: ClassColorSource;
  readonly classColorTarget: ClassColorTarget;
  readonly healingPrediction: HealingPredictionTarget | null;
  readonly barLossColor: string | null;
  readonly healingColor: string;
  readonly overhealColor: string;
  readonly overhealTexture: BarTexture;
  readonly overhealTextureSize: number;
  readonly barDirection: Direction | null;
  readonly fontFamily: FontFamily;
  readonly bold: boolean;
  readonly alignment: TextAlignment;
  readonly vertical: TextAnchor;
  readonly outline: boolean;
  readonly shadow: boolean;
  readonly wrap: boolean;
  readonly borderWidth: number;
  readonly radius: number;
  readonly imageFit: ImageFit;
  readonly desaturate: boolean;
  readonly glow: boolean;
  readonly glowColor: string;
  readonly animation: WidgetAnimation;
  readonly periodMs: number;
  readonly fillColor: string;
  readonly barValue: number;
  readonly barVertical: boolean;
  readonly barInverse: boolean;
  readonly outlineWidth: number;
  readonly outlineColor: string;
  readonly borderStyle: BorderStyle;
  readonly layer: number;
  readonly zoom: number;
  readonly swipe: boolean;
  readonly swipeReverse: boolean;
  readonly showTimer: boolean;
  readonly showStacks: boolean;
  readonly warnBelowSeconds: number;
  readonly timerPosition: TextPosition;
  readonly stacksPosition: TextPosition;
  readonly labelPosition: TextPosition;
  readonly glowMode: GlowMode;
  readonly gradient: string | null;
  readonly spark: boolean;
  readonly sparkColor: string;
  readonly ticks: number[];
  readonly tickColor: string;
  readonly rightText: string;
  readonly source: ProgressSource;
  readonly path: string;
  readonly rangeMin: number;
  readonly rangeMax: number;
  readonly transitionMs: number;
  readonly barIconEnabled: boolean;
  readonly barIconAnchor: BarIconAnchor;
  readonly barIconSize: number;
  readonly barIconX: number;
  readonly barIconY: number;
  readonly barIconPadding: number;
  readonly animationWhen: AnimationWhen;
  readonly animationIntensity: number;
  readonly animationThreshold: number;
  readonly animationDuration: number;
  readonly animationPath: string;
  readonly animationChange: AnimationChange;
  readonly showMotion: MotionKind;
  readonly hideMotion: MotionKind;
  readonly motionDuration: number;
  readonly imageSource: ImageSource;
  readonly imageSkillId: number | null;
  readonly imageEffectId: number | null;
  readonly animationColor: string;
  readonly backgroundOpacity: number | null;
}

export const defaultWidgetStyle: WidgetStyle = {
  classColorSource: ClassColorSource.None,
  classColorTarget: ClassColorTarget.Fill,
  healingPrediction: null,
  barLossColor: null,
  healingColor: '#a5e8ba99',
  overhealColor: '#54d985b3',
  overhealTexture: BarTexture.Solid,
  overhealTextureSize: 8,
  barDirection: null,
  fontFamily: FontFamily.Lora,
  bold: false,
  alignment: TextAlignment.Center,
  vertical: TextAnchor.Middle,
  outline: false,
  shadow: false,
  wrap: true,
  borderWidth: 1,
  radius: 0,
  imageFit: ImageFit.Cover,
  desaturate: false,
  glow: false,
  glowColor: '#d6b26e',
  animation: WidgetAnimation.None,
  periodMs: 1000,
  fillColor: '#d6b26e',
  barValue: 50,
  barVertical: false,
  barInverse: false,
  outlineWidth: 1,
  outlineColor: '#000000',
  borderStyle: BorderStyle.Solid,
  layer: 0,
  zoom: 0,
  swipe: true,
  swipeReverse: false,
  showTimer: true,
  showStacks: true,
  warnBelowSeconds: 0,
  timerPosition: TextPosition.Center,
  stacksPosition: TextPosition.BottomRight,
  labelPosition: TextPosition.Top,
  glowMode: GlowMode.None,
  gradient: null,
  spark: false,
  sparkColor: '#ffffff',
  ticks: [],
  tickColor: '#ffffff',
  rightText: '',
  source: ProgressSource.Fixed,
  path: 'player.hp.value',
  rangeMin: 0,
  rangeMax: 100,
  transitionMs: 0,
  barIconEnabled: false,
  barIconAnchor: BarIconAnchor.Left,
  barIconSize: 20,
  barIconX: 0,
  barIconY: 0,
  barIconPadding: 2,
  animationWhen: AnimationWhen.Always,
  animationIntensity: 75,
  animationThreshold: 5,
  animationDuration: 2000,
  animationPath: 'combat.damage',
  animationChange: AnimationChange.Any,
  showMotion: MotionKind.None,
  hideMotion: MotionKind.None,
  motionDuration: 250,
  imageSource: ImageSource.Automatic,
  imageSkillId: null,
  imageEffectId: null,
  animationColor: '#d6b26e',
  backgroundOpacity: null,
};
