import { useEffect, useRef } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { onRenderFrame } from "../lib/renderBridge";
import { getPixiApp, getPixiCanvas, destroyPixiApp } from "../renderer/PixiApp";
import { FrameRenderer } from "../renderer/FrameRenderer";
import { InputManager } from "../input/InputManager";

/** Game View — loads a WASM plugin, renders frames via PixiJS. */
export default function GameView() {
  const [params] = useSearchParams();
  const pluginId = params.get("plugin") ?? "";
  const navigate = useNavigate();
  const canvasRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<FrameRenderer | null>(null);
  const inputRef = useRef<InputManager | null>(null);

  useEffect(() => {
    if (!pluginId) return;

    let unlisten: (() => void) | undefined;
    let mounted = true;

    (async () => {
      // Initialize PixiJS
      const app = await getPixiApp();
      if (!mounted) return;

      const canvas = getPixiCanvas();
      if (canvas && canvasRef.current) {
        canvasRef.current.innerHTML = "";
        canvasRef.current.appendChild(canvas);
      }

      rendererRef.current = new FrameRenderer(app);

      // Initialize input capture
      inputRef.current = new InputManager();

      // Start the game
      await invoke("start_game", { pluginId });

      // Subscribe to render frames
      if (!mounted) return;
      unlisten = await onRenderFrame((frame) => {
        rendererRef.current?.renderFrame(frame);
      });
    })().catch((err) => console.error("start_game failed:", err));

    return () => {
      mounted = false;
      unlisten?.();
      inputRef.current?.destroy();
      inputRef.current = null;
      rendererRef.current?.destroy();
      rendererRef.current = null;
      destroyPixiApp();
      invoke("stop_game").catch(() => {});
    };
  }, [pluginId]);

  // ESC key → back to menu
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.code === "Escape") {
        e.preventDefault();
        handleBack();
      }
    };
    window.addEventListener("keydown", handleEsc);
    return () => window.removeEventListener("keydown", handleEsc);
  });

  const handleBack = () => {
    invoke("stop_game").catch(() => {});
    navigate("/");
  };

  return (
    <div style={{ paddingTop: "1rem" }}>
      <button onClick={handleBack} style={btnStyle}>
        ← Back to Menu
      </button>
      <div
        ref={canvasRef}
        style={{
          marginTop: "1rem",
          width: "960px",
          height: "720px",
          background: "#0a0a0f",
          borderRadius: "8px",
          overflow: "hidden",
        }}
      >
        {!pluginId && (
          <p style={{ color: "#555", textAlign: "center", paddingTop: "2rem" }}>
            No plugin selected — go back and pick a game
          </p>
        )}
      </div>
    </div>
  );
}

const btnStyle: React.CSSProperties = {
  background: "#222",
  color: "#fff",
  border: "1px solid #444",
  padding: "0.5rem 1rem",
  borderRadius: "6px",
  cursor: "pointer",
  fontSize: "0.9rem",
};
