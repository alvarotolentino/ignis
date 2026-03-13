import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** A single sound asset payload from Rust. */
interface SoundAsset {
  id: number;
  data_base64: string;
  mime: string;
}

/** Collection of sound assets from Rust. */
interface SoundsPayload {
  sounds: SoundAsset[];
}

/**
 * Manages sound playback using the Web Audio API.
 * Listens for `load_sounds` events emitted by the Rust backend.
 */
export class AudioManager {
  private audioCtx: AudioContext | null = null;
  private buffers = new Map<number, AudioBuffer>();
  private unlisten: UnlistenFn | null = null;

  /** Start listening for sound loading events. */
  async init(): Promise<void> {
    this.unlisten = await listen<SoundsPayload>("load_sounds", (event) => {
      this.loadSounds(event.payload).catch((err) =>
        console.error("Failed to load sounds:", err)
      );
    });
  }

  /** Lazily create the AudioContext (must be after user gesture). */
  private getAudioContext(): AudioContext {
    if (!this.audioCtx) {
      this.audioCtx = new AudioContext();
    }
    return this.audioCtx;
  }

  /** Decode and store all sound assets. */
  private async loadSounds(payload: SoundsPayload): Promise<void> {
    this.buffers.clear();
    const ctx = this.getAudioContext();

    for (const asset of payload.sounds) {
      try {
        const binary = atob(asset.data_base64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) {
          bytes[i] = binary.charCodeAt(i);
        }
        const buffer = await ctx.decodeAudioData(bytes.buffer);
        this.buffers.set(asset.id, buffer);
      } catch (err) {
        console.warn(`Failed to decode sound ${asset.id}:`, err);
      }
    }

    console.log(`Loaded ${this.buffers.size} sounds`);
  }

  /** Play a sound by ID. No-op if the sound is not loaded. */
  playSound(id: number): void {
    const buffer = this.buffers.get(id);
    if (!buffer) {
      console.warn(`Sound ${id} not loaded`);
      return;
    }

    const ctx = this.getAudioContext();
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(ctx.destination);
    source.start(0);
  }

  /** Clean up all resources. */
  destroy(): void {
    this.unlisten?.();
    this.unlisten = null;
    this.buffers.clear();
    if (this.audioCtx) {
      this.audioCtx.close().catch(() => {});
      this.audioCtx = null;
    }
  }
}
