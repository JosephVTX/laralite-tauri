export function fmtBytes(n: number | null): string {
  if (!n) return "—";
  return n >= 1e9 ? `${(n / 1e9).toFixed(1)} GB` : n >= 1e6 ? `${(n / 1e6).toFixed(1)} MB` : `${Math.round(n / 1e3)} KB`;
}

export const fmtDate = (unixSecs: number) => new Date(unixSecs * 1000).toLocaleString("es");
