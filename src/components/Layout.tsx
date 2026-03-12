import { ReactNode } from "react";

interface LayoutProps {
  children: ReactNode;
}

/** Full-viewport dark container for all pages. */
export default function Layout({ children }: LayoutProps) {
  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        background: "#0a0a0f",
        color: "#ffffff",
        fontFamily: "monospace",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        overflow: "hidden",
      }}
    >
      <div style={{ width: "100%", maxWidth: 960, flex: 1, padding: "1rem" }}>
        {children}
      </div>
    </div>
  );
}
