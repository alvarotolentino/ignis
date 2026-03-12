import { Application, Container, Graphics, Text, TextStyle } from "pixi.js";
import type { RenderFrame, RenderCommand } from "../lib/renderBridge";

/**
 * Renders RenderFrame data onto a PixiJS stage.
 * Each frame clears the game container and recreates display objects
 * from the command list. Simple and correct — optimize later if needed.
 */
export class FrameRenderer {
  private gameLayer: Container;

  constructor(private app: Application) {
    this.gameLayer = new Container();
    this.app.stage.addChild(this.gameLayer);
  }

  /** Render a full frame of commands. Clears previous frame first. */
  renderFrame(frame: RenderFrame): void {
    // Remove all children from previous frame
    this.gameLayer.removeChildren();

    for (const cmd of frame.commands) {
      switch (cmd.type) {
        case "DrawRect":
          this.drawRect(cmd);
          break;
        case "DrawText":
          this.drawText(cmd);
          break;
        case "DrawSprite":
          // Sprite rendering not yet implemented — skip silently
          break;
      }
    }
  }

  private drawRect(cmd: RenderCommand): void {
    const { hex, alpha } = rgbaToPixi(cmd.color ?? 0xffffffff);
    const g = new Graphics();
    g.rect(cmd.x ?? 0, cmd.y ?? 0, cmd.w ?? 0, cmd.h ?? 0);
    g.fill({ color: hex, alpha });
    this.gameLayer.addChild(g);
  }

  private drawText(cmd: RenderCommand): void {
    const style = new TextStyle({
      fontSize: cmd.size ?? 16,
      fill: 0xffffff,
      fontFamily: "monospace",
    });
    const t = new Text({ text: cmd.text ?? "", style });
    t.x = cmd.x ?? 0;
    t.y = cmd.y ?? 0;
    this.gameLayer.addChild(t);
  }

  /** Clean up the game layer. */
  destroy(): void {
    this.gameLayer.destroy({ children: true });
  }
}

/**
 * Convert a u32 RGBA color to PixiJS hex + alpha.
 * Input: 0xRRGGBBAA  →  hex: 0xRRGGBB, alpha: AA/255
 */
function rgbaToPixi(color: number): { hex: number; alpha: number } {
  const hex = (color >>> 8) & 0xffffff;
  const alpha = (color & 0xff) / 255;
  return { hex, alpha };
}
