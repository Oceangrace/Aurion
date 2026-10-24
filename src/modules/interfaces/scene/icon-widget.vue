<script setup lang="ts">
import { ImageFit } from '../model/image-fit';
import { inject, computed, onBeforeUnmount, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { WidgetNode } from '../model/widget-node';
import type { PreviewVisual } from '../model/preview-visual';
import { IconAssets } from '../../../renderer/assets/icon-assets';
import { iconAssetsContextKey } from '../../../renderer/assets/icon-assets-context-key';
import PositionedText from './positioned-text.vue';
import { timerClock } from './timer-clock';
const props = withDefaults(
  defineProps<{
    widget: WidgetNode;
    runtime?: PreviewVisual | null;
    ticking?: boolean;
    sampledAt?: number | null;
  }>(),
  { runtime: null, ticking: false, sampledAt: null },
);

const { t } = useI18n();

const assets = inject(iconAssetsContextKey, () => new IconAssets(), true);

const missing = ref(false);

const observedEffect = computed(() => (props.runtime?.stacks ?? 0) > 0);

const url = computed(
  () => props.widget.appearance.customImage ?? assets.url(props.widget.appearance.iconPath),
);

watch(url, () => {
  missing.value = false;
});
const now = timerClock.current();

const received = ref(props.sampledAt ?? timerClock.snapshot());

const counting = computed(
  () => props.ticking && props.runtime !== null && (props.runtime.remaining ?? 0) > 0,
);

watch(
  counting,
  (value, previous) => {
    if (value === previous) {
      return;
    }

    if (value === true) {
      timerClock.acquire();
    } else if (previous === true) {
      timerClock.release();
    }
  },
  { immediate: true },
);

watch(
  () => props.runtime,
  () => {
    received.value = props.sampledAt ?? timerClock.snapshot();
  },
);

onBeforeUnmount(() => {
  if (counting.value === true) {
    timerClock.release();
  }
});

const remaining = computed(() => {
  const value = props.runtime?.remaining ?? null;

  if (value === null || counting.value === false) {
    return value;
  }

  return Math.max(0, value - (now.value - received.value) / 1000);
});

const duration = computed(() => props.runtime?.duration ?? null);

const fraction = computed(() =>
  remaining.value === null || duration.value === null || duration.value <= 0
    ? null
    : Math.max(0, Math.min(1, remaining.value / duration.value)),
);

const swipe = computed(() => {
  const value = props.runtime?.remaining ?? null;

  const total = duration.value;

  if (counting.value === false || value === null || total === null || total <= 0 === true) {
    return null;
  }

  const angle = (left: number): number => {
    const share = Math.max(0, Math.min(1, left / total));

    return (props.widget.style.swipeReverse ? 1 - share : share) * 360;
  };

  return {
    '--swipe-from': String(angle(value)) + 'deg',
    '--swipe-to': String(angle(0)) + 'deg',
    '--swipe-duration': String(value) + 's',
    '--swipe-delay': String(-(performance.now() - received.value) / 1000) + 's',
  };
});

const timer = computed(() => {
  const seconds = remaining.value;

  if (seconds === null || seconds <= 0 === true) {
    return '';
  }

  if (seconds < 10 === true) {
    return seconds.toFixed(1);
  }

  return seconds < 60
    ? String(Math.floor(seconds))
    : String(Math.floor(seconds / 60)) + ':' + String(Math.floor(seconds) % 60).padStart(2, '0');
});
</script>
<template>
  <span class="icon-artwork" :style="{ borderRadius: String(widget.style.radius) + 'px' }">
    <img
      v-if="url !== null && !missing"
      :src="url"
      :alt="widget.name"
      draggable="false"
      :style="{
        objectFit: widget.style.imageFit === ImageFit.Stretch ? 'fill' : widget.style.imageFit,
        filter: widget.style.desaturate ? 'grayscale(1)' : 'none',
        transform: 'scale(' + String(1 + widget.style.zoom / 100) + ')',
      }"
      @error="missing = true"
    />
    <span
      v-else
      class="widget-icon-empty"
      :class="{ 'effect-icon-fallback': observedEffect }"
      :title="
        observedEffect
          ? t('modules.interfaces.effectIconUnknown')
          : t('modules.interfaces.missingIcon')
      "
      ><span aria-hidden="true">{{ observedEffect ? '!' : '◇' }}</span
      ><small v-if="observedEffect === false">{{
        t('modules.interfaces.missingIcon')
      }}</small></span
    >
    <span
      v-if="widget.style.swipe && swipe !== null"
      :key="received"
      class="icon-swipe icon-swipe-running"
      :style="swipe"
    />
    <span
      v-else-if="widget.style.swipe && fraction !== null"
      class="icon-swipe"
      :style="{
        background:
          'conic-gradient(#080b12ba ' +
          String((widget.style.swipeReverse ? 1 - fraction : fraction) * 360) +
          'deg, transparent 0)',
      }"
    />
    <span v-if="widget.style.borderWidth > 0" class="icon-bevel" />
  </span>
  <PositionedText
    v-if="widget.appearance.text !== ''"
    :position="widget.style.labelPosition"
    :text="widget.appearance.text"
  />
  <PositionedText
    v-if="widget.style.showTimer && timer !== ''"
    class="icon-timer"
    :position="widget.style.timerPosition"
    :text="timer"
    single-line
    :style="{
      color:
        widget.style.warnBelowSeconds > 0 &&
        remaining !== null &&
        remaining <= widget.style.warnBelowSeconds
          ? '#ffcf70'
          : undefined,
    }"
  />
  <PositionedText
    v-if="widget.style.showStacks && (runtime?.stacks ?? 0) > 1"
    class="icon-stacks"
    :position="widget.style.stacksPosition"
    :text="String(runtime?.stacks)"
    single-line
  />
  <span
    v-if="widget.style.glowMode !== 'none'"
    class="icon-proc-glow"
    :class="'glow-' + widget.style.glowMode"
    :style="{
      '--glow-color': widget.style.glowColor,
      borderRadius: String(widget.style.radius) + 'px',
    }"
  />
</template>
<style scoped>
.icon-artwork {
  position: absolute;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
}
.icon-artwork img {
  display: block;
  width: 100%;
  height: 100%;
}
.icon-bevel {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  box-shadow:
    inset 0 0 0 1px #000b,
    inset 0 2px 0 1px #ffffff38,
    inset 0 -2px 0 1px #0008;
}
.icon-timer,
.icon-stacks {
  font-weight: 700;
  text-shadow:
    -1px -1px 0 #080b12,
    1px -1px 0 #080b12,
    -1px 1px 0 #080b12,
    1px 1px 0 #080b12,
    0 2px 3px #000;
}
.icon-stacks {
  padding: 0 2px;
  border-radius: 2px;
  background: #080b1266;
}
.effect-icon-fallback {
  color: #e7b2aa;
  background: #25201f;
  font-size: 0.9em;
}
.icon-swipe {
  position: absolute;
  inset: 0;
}
@property --icon-swipe-angle {
  syntax: '<angle>';
  inherits: false;
  initial-value: 0deg;
}
.icon-swipe-running {
  background: conic-gradient(#080b12ba var(--icon-swipe-angle), transparent 0);
  animation: icon-swipe var(--swipe-duration) linear var(--swipe-delay) both;
}
@keyframes icon-swipe {
  from {
    --icon-swipe-angle: var(--swipe-from);
  }
  to {
    --icon-swipe-angle: var(--swipe-to);
  }
}
.icon-proc-glow {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 1;
}
.glow-proc {
  inset: -2px;
  border: 1px solid color-mix(in srgb, var(--glow-color), white 55%);
  box-shadow:
    0 0 4px 1px var(--glow-color),
    0 0 12px 2px color-mix(in srgb, var(--glow-color) 65%, transparent),
    inset 0 0 5px 1px var(--glow-color);
}
.glow-proc::before,
.glow-proc::after {
  content: '';
  position: absolute;
  pointer-events: none;
  border-radius: inherit;
}
.glow-proc::before {
  inset: -3px;
  padding: 2px;
  background: conic-gradient(
    from var(--icon-proc-angle),
    transparent 0deg,
    var(--glow-color) 50deg,
    #fff9d9 85deg,
    transparent 100deg 180deg,
    var(--glow-color) 230deg,
    #fff9d9 265deg,
    transparent 280deg
  );
  mask:
    linear-gradient(#fff 0 0) content-box,
    linear-gradient(#fff 0 0);
  mask-composite: exclude;
  animation: icon-proc-orbit 1800ms linear infinite;
}
.glow-proc::after {
  inset: 0;
  box-shadow: inset 0 0 8px 2px var(--glow-color);
  animation: icon-proc 1100ms ease-in-out infinite alternate;
}
@property --icon-proc-angle {
  syntax: '<angle>';
  inherits: false;
  initial-value: 0deg;
}
@keyframes icon-proc-orbit {
  to {
    --icon-proc-angle: 360deg;
  }
}
.glow-pixel {
  background:
    linear-gradient(90deg, var(--glow-color) 50%, transparent 0) repeat-x top / 8px 2px,
    linear-gradient(90deg, var(--glow-color) 50%, transparent 0) repeat-x bottom / 8px 2px,
    linear-gradient(0deg, var(--glow-color) 50%, transparent 0) repeat-y left / 2px 8px,
    linear-gradient(0deg, var(--glow-color) 50%, transparent 0) repeat-y right / 2px 8px;
  animation: icon-ants 500ms linear infinite;
}
@keyframes icon-proc {
  from {
    opacity: 0.3;
  }
  to {
    opacity: 1;
  }
}
@keyframes icon-ants {
  to {
    background-position:
      8px 0,
      -8px 100%,
      0 -8px,
      100% 8px;
  }
}
@media (prefers-reduced-motion: reduce) {
  .icon-proc-glow,
  .icon-proc-glow::before,
  .icon-proc-glow::after {
    animation: none;
  }
}
</style>
