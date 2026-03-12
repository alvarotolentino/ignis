import { Application } from "pixi.js";

let app: Application | null = null;
let initPromise: Promise<Application> | null = null;

/**
 * Returns the singleton PixiJS Application, creating it on first call.
 * Must call `await getPixiApp()` before use since PixiJS v8 init is async.
 * Safe to call multiple times — deduplicates concurrent init calls.
 */
export async function getPixiApp(): Promise<Application> {
  if (app) return app;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    const instance = new Application();
    await instance.init({
      width: 960,
      height: 720,
      backgroundColor: 0x0a0a0f,
      antialias: false,
    });

    // Pixel-perfect rendering
    instance.renderer.resolution = 1;

    app = instance;
    initPromise = null;
    return instance;
  })();

  return initPromise;
}

/** Returns the PixiJS canvas element (call after getPixiApp). */
export function getPixiCanvas(): HTMLCanvasElement | null {
  return app?.canvas ?? null;
}

/**
 * Destroys the singleton app and resets the reference.
 * Safe to call even if init hasn't completed yet.
 */
export function destroyPixiApp(): void {
  // Cancel any in-flight init so it won't set `app` after we clear it
  initPromise = null;

  if (app) {
    try {
      app.destroy(true, { children: true, texture: true });
    } catch {
      // PixiJS v8 can throw if destroy() is called on a partially-initialized
      // app (e.g., React Strict Mode unmounts before init completes).
      // Safe to swallow — we're tearing down anyway.
    }
    app = null;
  }
}
