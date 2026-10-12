import { memo } from "react";

export type PaneId = "subs" | "hablantes" | "estilos";

export interface PaneTab {
  id: PaneId;
  label: string;
  title: string;
  contador: number | null;
}

interface Props {
  panes: PaneTab[];
  activo: PaneId;
  onCambiar: (id: PaneId) => void;
}

// Glifos por pane (SVG inline currentColor, nunca emoji): mismo trazo que el
// resto de la app (viewBox 16, stroke 1.4-1.6, clase icon xs).
function iconoPane(id: PaneId) {
  if (id === "hablantes") {
    return (
      <svg
        className="icon xs"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        strokeWidth={1.4}
        strokeLinecap="round"
        aria-hidden="true"
      >
        <circle cx="6" cy="5.5" r="2.6" />
        <path d="M1.8 13.5c0-2.5 1.9-4 4.2-4s4.2 1.5 4.2 4" />
        <circle cx="11.5" cy="6" r="2.2" />
        <path d="M11 9.7c2.1.2 3.4 1.6 3.4 3.8" />
      </svg>
    );
  }
  if (id === "estilos") {
    return (
      <svg
        className="icon xs"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        strokeWidth={1.4}
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="M8 2.5c2.5 3 4.5 5.3 4.5 7.5a4.5 4.5 0 0 1-9 0C3.5 7.8 5.5 5.5 8 2.5z" />
      </svg>
    );
  }
  return (
    <svg
      className="icon xs"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.4}
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="M2 4h12M2 8h12M2 12h7" />
    </svg>
  );
}

function PanelTabs({ panes, activo, onCambiar }: Props) {
  return (
    <div className="panelTabs" role="tablist">
      {panes.map((p) => (
        <button
          key={p.id}
          type="button"
          role="tab"
          aria-selected={p.id === activo}
          className={"panelTab" + (p.id === activo ? " activo" : "")}
          title={p.title}
          onClick={() => onCambiar(p.id)}
        >
          {iconoPane(p.id)}
          <span>{p.label}</span>
          {p.contador !== null && p.contador > 0 && (
            <span className="tabCounterBadge">{p.contador}</span>
          )}
        </button>
      ))}
    </div>
  );
}

export default memo(PanelTabs);
