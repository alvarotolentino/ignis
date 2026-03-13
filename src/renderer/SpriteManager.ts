import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Assets, Texture, Rectangle } from "pixi.js";

/** Sprite sheet metadata payload from Rust. */
interface SpriteSheetPayload {
  image_base64: string;
  meta_json: string;
}

/** Frame entry in the spritesheet JSON. */
interface SpriteFrame {
  frame: { x: number; y: number; w: number; h: number };
}

/** Spritesheet JSON format (simplified TexturePacker). */
interface SpriteSheetMeta {
  frames: Record<string, SpriteFrame>;
}

/**
 * Manages sprite textures loaded from a plugin's sprite sheet.
 * Listens for `load_sprite_sheet` events emitted by the Rust backend.
 */
export class SpriteManager {
  private sprites = new Map<number, Texture>();
  private unlisten: UnlistenFn | null = null;
  private baseTexture: Texture | null = null;

  /** Start listening for sprite sheet events. */
  async init(): Promise<void> {
    this.unlisten = await listen<SpriteSheetPayload>(
      "load_sprite_sheet",
      (event) => {
        this.loadSpriteSheet(event.payload).catch((err) =>
          console.error("Failed to load sprite sheet:", err)
        );
      }
    );
  }

  /** Load a sprite sheet from a base64-encoded PNG + JSON metadata. */
  private async loadSpriteSheet(payload: SpriteSheetPayload): Promise<void> {
    // Clear any previously loaded sprites
    this.clear();

    const dataUrl = `data:image/png;base64,${payload.image_base64}`;

    // Load the full atlas texture
    this.baseTexture = await Assets.load<Texture>(dataUrl);

    // Parse the frame metadata
    const meta: SpriteSheetMeta = JSON.parse(payload.meta_json);

    // Create sub-textures for each frame
    for (const [key, entry] of Object.entries(meta.frames)) {
      const id = parseInt(key, 10);
      if (isNaN(id)) continue;

      const { x, y, w, h } = entry.frame;
      const subTexture = new Texture({
        source: this.baseTexture.source,
        frame: new Rectangle(x, y, w, h),
      });

      this.sprites.set(id, subTexture);
    }

    console.log(`Loaded ${this.sprites.size} sprites from sprite sheet`);
  }

  /** Get a sprite texture by ID. Returns undefined if not loaded. */
  getSprite(id: number): Texture | undefined {
    return this.sprites.get(id);
  }

  /** Clear all loaded sprites. */
  clear(): void {
    this.sprites.clear();
    this.baseTexture = null;
  }

  /** Stop listening and clean up. */
  destroy(): void {
    this.unlisten?.();
    this.unlisten = null;
    this.clear();
  }
}
