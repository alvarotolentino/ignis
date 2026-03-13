import { useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { getHighScores, type ScoreEntry } from "../lib/tauri";

/** Medal colours for top 3 ranks. */
const MEDAL_COLORS = ["#FFD700", "#C0C0C0", "#CD7F32"] as const;

/** Displays the top-10 high scores for a given game. */
export default function HighScores() {
  const [params] = useSearchParams();
  const gameId = params.get("game") ?? "";
  const navigate = useNavigate();
  const [scores, setScores] = useState<ScoreEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!gameId) return;
    getHighScores(gameId, 10)
      .then(setScores)
      .catch((err) => console.error("Failed to load scores:", err))
      .finally(() => setLoading(false));
  }, [gameId]);

  return (
    <div style={{ paddingTop: "2rem", maxWidth: 600, margin: "0 auto" }}>
      <h2 style={{ textAlign: "center", marginBottom: "1.5rem" }}>
        High Scores — {gameId || "Unknown"}
      </h2>

      {loading ? (
        <p style={{ color: "#888", textAlign: "center" }}>Loading…</p>
      ) : scores.length === 0 ? (
        <p style={{ color: "#888", textAlign: "center" }}>
          No scores recorded yet. Play a game first!
        </p>
      ) : (
        <table style={tableStyle}>
          <thead>
            <tr>
              <th style={thStyle}>#</th>
              <th style={{ ...thStyle, textAlign: "left" }}>Player</th>
              <th style={thStyle}>Score</th>
              <th style={thStyle}>Date</th>
            </tr>
          </thead>
          <tbody>
            {scores.map((entry, i) => {
              const color = i < 3 ? MEDAL_COLORS[i] : "#ffffff";
              return (
                <tr key={i} style={{ color }}>
                  <td style={tdStyle}>{i + 1}</td>
                  <td style={{ ...tdStyle, textAlign: "left" }}>
                    {entry.player_name}
                  </td>
                  <td style={tdStyle}>{entry.score.toLocaleString()}</td>
                  <td style={tdStyle}>
                    {entry.achieved_at.split("T")[0] ?? entry.achieved_at}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}

      <div style={{ textAlign: "center", marginTop: "2rem" }}>
        <button onClick={() => navigate("/")} style={btnStyle}>
          Back to Menu
        </button>
      </div>
    </div>
  );
}

const tableStyle: React.CSSProperties = {
  width: "100%",
  borderCollapse: "collapse",
  fontFamily: "monospace",
};

const thStyle: React.CSSProperties = {
  padding: "0.5rem 1rem",
  borderBottom: "2px solid #444",
  textAlign: "center",
  color: "#aaa",
  fontSize: "0.85rem",
};

const tdStyle: React.CSSProperties = {
  padding: "0.5rem 1rem",
  borderBottom: "1px solid #222",
  textAlign: "center",
  fontSize: "0.95rem",
};

const btnStyle: React.CSSProperties = {
  background: "#222",
  color: "#fff",
  border: "1px solid #444",
  padding: "0.5rem 1rem",
  borderRadius: "6px",
  cursor: "pointer",
  fontSize: "0.9rem",
};
