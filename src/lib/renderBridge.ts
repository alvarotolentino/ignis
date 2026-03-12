import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface RenderCommand {
  type: 'DrawRect' | 'DrawSprite' | 'DrawText';
  // DrawRect fields
  x?: number;
  y?: number;
  w?: number;
  h?: number;
  color?: number;
  // DrawSprite fields
  id?: number;
  // DrawText fields
  text?: string;
  size?: number;
}

export interface RenderFrame {
  commands: RenderCommand[];
}

/**
 * Subscribe to `render_frame` events emitted by the Rust game loop.
 * Returns an unlisten function to unsubscribe.
 */
export function onRenderFrame(
  callback: (frame: RenderFrame) => void
): Promise<UnlistenFn> {
  return listen<RenderFrame>('render_frame', (event) => {
    callback(event.payload);
  });
}
