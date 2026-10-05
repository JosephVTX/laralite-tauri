import { useToasts } from "../lib/stores";

export function Toasts() {
  const items = useToasts((s) => s.items);
  return (
    <div className="toast toast-end z-50">
      {items.map((t) => (
        <div key={t.id} className={`alert alert-${t.kind} max-w-md break-words`}>
          <span className="whitespace-pre-wrap">{t.text}</span>
        </div>
      ))}
    </div>
  );
}
