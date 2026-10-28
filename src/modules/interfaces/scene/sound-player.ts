import { shallowRef, triggerRef } from 'vue';
import type { DesktopCommands } from '../../../core/desktop/desktop-commands';
import type { SpeechVoice } from '../model/speech-voice';
import type { SoundTone } from './sound-tone';
import type { SoundPlayback } from './sound-playback';
import { SoundCue } from '../model/sound-cue';
import { SoundLibrary } from './sound-library';

export class SoundPlayer {
  public readonly playing = shallowRef<string | null>(null);
  public readonly error = shallowRef<string | null>(null);
  public readonly voices = shallowRef<readonly SpeechVoice[]>([]);
  private context: AudioContext | null = null;
  private readonly library = new SoundLibrary();
  private readonly cache = shallowRef(new Map<string, AudioBuffer>());
  private readonly pending = new Map<string, Promise<AudioBuffer>>();
  private voicesRequest: Promise<void> | null = null;
  private preparation = 0;
  private prepareTimer: ReturnType<typeof setTimeout> | null = null;
  private generation = 0;
  private sources: AudioScheduledSourceNode[] = [];
  private timer: ReturnType<typeof setTimeout> | null = null;

  public constructor(private readonly desktop: DesktopCommands) {}

  public async loadVoices(): Promise<void> {
    this.voicesRequest ??= this.readVoices();

    return this.voicesRequest;
  }

  public prepare(requests: readonly SoundPlayback[]): void {
    const generation = ++this.preparation;

    if (this.prepareTimer !== null) {
      clearTimeout(this.prepareTimer);
    }

    const speech = requests.filter(
      (request) => request.cue === SoundCue.Speech && this.audible(request),
    );

    const unique = new Map(speech.map((request) => [this.cacheKey(request), request]));

    this.prepareTimer = setTimeout(() => {
      this.prepareTimer = null;
      void this.warm([...unique.values()].slice(0, 32), generation);
    }, 150);
  }

  public isReady(request: SoundPlayback): boolean {
    return request.cue !== SoundCue.Speech || this.cache.value.has(this.cacheKey(request));
  }

  public async prepareSpeech(request: SoundPlayback): Promise<void> {
    if (request.cue !== SoundCue.Speech || this.audible(request) === false) {
      return;
    }

    try {
      this.context ??= new AudioContext();
      await this.speechBuffer(this.context, request);
    } catch (error) {
      this.error.value = String(error);
    }
  }

  public async play(request: SoundPlayback): Promise<void> {
    if (this.audible(request) === false) {
      return;
    }

    this.stop();
    this.error.value = null;

    const generation = this.generation;

    this.playing.value = request.owner;

    try {
      this.context ??= new AudioContext();
      await this.context.resume();

      if (generation !== this.generation) {
        return;
      }

      const duration = await this.duration(this.context, request, generation);

      if (generation === this.generation) {
        this.timer = setTimeout(
          () => {
            this.stop();
          },
          (duration + 0.05) * 1000,
        );
      }
    } catch (error) {
      if (generation === this.generation) {
        this.error.value = String(error);
        this.stop();
      }
    }
  }

  public stop(): void {
    this.generation++;
    this.playing.value = null;

    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }

    for (const source of this.sources) {
      source.stop();
      source.disconnect();
    }

    this.sources = [];
  }

  public dispose(): void {
    this.preparation++;

    if (this.prepareTimer !== null) {
      clearTimeout(this.prepareTimer);
      this.prepareTimer = null;
    }

    this.stop();
    this.cache.value.clear();
    this.pending.clear();

    if (this.context !== null) {
      void this.context.close();
      this.context = null;
    }
  }

  private async readVoices(): Promise<void> {
    try {
      this.voices.value = await this.desktop.invoke<SpeechVoice[]>('interfaces_speech_voices');
    } catch (error) {
      this.error.value = String(error);
      this.voicesRequest = null;
    }
  }

  private async warm(requests: readonly SoundPlayback[], generation: number): Promise<void> {
    for (const request of requests) {
      if (generation !== this.preparation) {
        return;
      }

      await this.prepareSpeech(request);
    }
  }

  private cacheKey(request: SoundPlayback): string {
    return JSON.stringify([request.text?.trim() ?? '', request.voice ?? '', request.rate ?? 100]);
  }

  private audible(request: SoundPlayback): boolean {
    return (
      request.cue !== SoundCue.None &&
      request.volume > 0 &&
      (request.cue !== SoundCue.Speech || (request.text?.trim() ?? '') !== '')
    );
  }

  private signal(context: AudioContext, request: SoundPlayback): number {
    const tones = this.library.tones[request.cue];

    for (const tone of tones) {
      this.beep(context, tone, request.volume / 100);
    }

    return Math.max(0, ...tones.map((tone) => tone.startS + tone.lengthS));
  }

  private async speech(
    context: AudioContext,
    request: SoundPlayback,
    generation: number,
  ): Promise<number> {
    const buffer = await this.speechBuffer(context, request);

    if (generation !== this.generation) {
      return 0;
    }

    const source = context.createBufferSource();

    const gain = context.createGain();

    source.buffer = buffer;
    gain.gain.value = Math.min(1, Math.max(0, request.volume / 100));
    source.connect(gain).connect(context.destination);

    source.onended = (): void => {
      source.disconnect();
      gain.disconnect();
    };

    this.sources.push(source);
    source.start();

    return buffer.duration;
  }

  private async duration(
    context: AudioContext,
    request: SoundPlayback,
    generation: number,
  ): Promise<number> {
    return request.cue === SoundCue.Speech
      ? this.speech(context, request, generation)
      : this.signal(context, request);
  }

  private async speechBuffer(context: AudioContext, request: SoundPlayback): Promise<AudioBuffer> {
    const text = request.text?.trim() ?? '';

    if (text === '') {
      throw new Error('Enter a phrase to speak');
    }

    const key = this.cacheKey(request);

    const cached = this.cache.value.get(key);

    if (cached !== undefined) {
      return cached;
    }

    let pending = this.pending.get(key);

    if (pending === undefined) {
      pending = this.render(context, request, text)
        .then((buffer) => this.remember(key, buffer))
        .finally(() => this.pending.delete(key));
      this.pending.set(key, pending);
    }

    return pending;
  }

  private remember(key: string, buffer: AudioBuffer): AudioBuffer {
    if (this.cache.value.size >= 32 === true) {
      const oldest = this.cache.value.keys().next().value;

      if (oldest !== undefined) {
        this.cache.value.delete(oldest);
      }
    }

    this.cache.value.set(key, buffer);
    triggerRef(this.cache);

    return buffer;
  }

  private async render(
    context: AudioContext,
    request: SoundPlayback,
    text: string,
  ): Promise<AudioBuffer> {
    const encoded = await this.desktop.invoke<string>('interfaces_speech_render', {
      request: { text, voice: request.voice ?? '', rate: request.rate ?? 100 },
    });

    const bytes = Uint8Array.from(atob(encoded), (character) => character.charCodeAt(0));

    return context.decodeAudioData(bytes.buffer);
  }

  private beep(context: AudioContext, tone: SoundTone, volume: number): void {
    const oscillator = context.createOscillator();

    const gain = context.createGain();

    const start = context.currentTime + tone.startS;

    const end = start + tone.lengthS;

    oscillator.type = tone.wave ?? 'sine';
    oscillator.frequency.setValueAtTime(tone.frequency, start);

    if (tone.endFrequency !== undefined) {
      oscillator.frequency.exponentialRampToValueAtTime(tone.endFrequency, end);
    }

    gain.gain.setValueAtTime(0.0001, start);
    gain.gain.exponentialRampToValueAtTime(
      Math.max(0.0001, 0.22 * volume * (tone.gain ?? 1)),
      start + 0.008,
    );
    gain.gain.exponentialRampToValueAtTime(0.0001, end);
    oscillator.connect(gain).connect(context.destination);

    oscillator.onended = (): void => {
      oscillator.disconnect();
      gain.disconnect();
    };

    this.sources.push(oscillator);
    oscillator.start(start);
    oscillator.stop(end + 0.02);
  }
}
