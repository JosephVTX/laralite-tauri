import { create } from "zustand";
import type { ToolId } from "./api";

export interface Progress {
  tool: ToolId;
  id: string;
  phase: "downloading" | "extracting" | "done";
  downloaded: number;
  total: number;
}

export const progressKey = (tool: ToolId, id: string) => `${tool}:${id}`;

interface ProgressStore {
  byKey: Record<string, Progress>;
  set: (p: Progress) => void;
  clear: (tool: ToolId, id: string) => void;
}

export const useProgress = create<ProgressStore>((set) => ({
  byKey: {},
  set: (p) => set((s) => ({ byKey: { ...s.byKey, [progressKey(p.tool, p.id)]: p } })),
  clear: (tool, id) =>
    set((s) => {
      const { [progressKey(tool, id)]: _, ...rest } = s.byKey;
      return { byKey: rest };
    }),
}));

export interface Toast {
  id: number;
  kind: "success" | "error" | "info";
  text: string;
}

interface ToastStore {
  items: Toast[];
  push: (kind: Toast["kind"], text: string) => void;
}

let nextId = 1;
export const useToasts = create<ToastStore>((set) => ({
  items: [],
  push: (kind, text) => {
    const id = nextId++;
    set((s) => ({ items: [...s.items, { id, kind, text }] }));
    setTimeout(() => set((s) => ({ items: s.items.filter((t) => t.id !== id) })), 5000);
  },
}));

export type PageId = "services" | "binaries" | "extensions" | "env";

/** Navegacion compartida: permite saltar a una pagina (y herramienta) desde cualquier vista. */
export const useNav = create<{ page: PageId; tool: ToolId; go: (page: PageId, tool?: ToolId) => void }>((set) => ({
  page: "services",
  tool: "php",
  go: (page, tool) => set((s) => ({ page, tool: tool ?? s.tool })),
}));
