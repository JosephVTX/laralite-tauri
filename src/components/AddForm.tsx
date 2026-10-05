import { useState, type FormEvent } from "react";

/** Formulario de una fila con N campos de texto; reutilizado por PATH y variables. */
export function AddForm({ fields, busy, onSubmit }: { fields: string[]; busy?: boolean; onSubmit: (values: string[]) => void }) {
  const [values, setValues] = useState<string[]>(() => fields.map(() => ""));
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (values.some((v) => !v.trim())) return;
    onSubmit(values.map((v) => v.trim()));
    setValues(fields.map(() => ""));
  };
  return (
    <form onSubmit={submit} className="join w-full">
      {fields.map((placeholder, i) => (
        <input
          key={placeholder}
          className="input input-sm join-item w-full"
          placeholder={placeholder}
          value={values[i]}
          onChange={(e) => setValues(values.map((v, j) => (j === i ? e.target.value : v)))}
        />
      ))}
      <button className="btn btn-sm btn-primary join-item" disabled={busy}>
        Agregar
      </button>
    </form>
  );
}
