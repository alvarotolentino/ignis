import { invoke } from "@tauri-apps/api/core";

// ── Player types ─────────────────────────────────────────────

export interface Player {
  id: number;
  name: string;
  avatar_id: number;
  created_at: string;
}

export const listPlayers = () => invoke<Player[]>("list_players");

export const createPlayer = (name: string, avatarId: number) =>
  invoke<Player>("create_player", { name, avatarId });

export const deletePlayer = (id: number) =>
  invoke<void>("delete_player", { id });

// ── Plugin / Game types ──────────────────────────────────────

export interface PluginManifest {
  game: {
    name: string;
    version: string;
    author: string;
    igi_version: string;
  };
  display?: { resolution?: { width: number; height: number } };
  rendering?: { tier?: string };
}

export interface DiscoveredPlugin {
  id: string;
  manifest: PluginManifest;
}

export const listPlugins = () =>
  invoke<DiscoveredPlugin[]>("list_discovered_plugins");
