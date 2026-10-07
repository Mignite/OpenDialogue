import { useState } from "react";
import type { Hablante, PresetAss } from "../types";
import { useLocale } from "../i18n";

interface Props {
  presets: PresetAss[];
  hablantes: Hablante[];
  onCerrar: () => void;
  onExportar: (asignacion: Record<string, string>, presetSinHablante: string) => void;
}

/** Modal de export .ass: SOLO asignación hablante→estilo + exportar.
 *  Los estilos se gestionan en el panel de Estilos (nivel usuario, siempre
 *  disponible), así que acá no hay edición ni borrador que perder: la trampa
 *  del "Guardar antes de Exportar" desaparece por construcción. */
export function AssExportModal({
  presets,
  hablantes,
  onCerrar,
  onExportar,
}: Props) {
  const { t } = useLocale();
  // Mapeo hablante -> preset para este export. Parte del preset guardado en
  // cada hablante (`h.presetId`, dato del proyecto) y se puede cambiar acá
  // solo para esta exportación sin tocar el proyecto.
  const primero = presets[0]?.id ?? "";
  const [asignacion, setAsignacion] = useState<Record<string, string>>({});
  const [presetSinHablante, setPresetSinHablante] = useState<string>(primero);

  const presetDe = (hablanteId: string): string => {
    const h = hablantes.find((x) => x.id === hablanteId);
    return asignacion[hablanteId] || h?.presetId || primero;
  };

  return (
    <div className="assOverlay" onClick={onCerrar}>
      <div className="assModal" onClick={(e) => e.stopPropagation()}>
        <div className="assModalHead">
          <h2>{t("assExport.title")}</h2>
          <span className="grow" />
          <span className="muted">{t("assExport.assignmentHint")}</span>
        </div>

        <div className="assAssign">
          <div className="assAssignHead">
            <span className="assAssignTitle">
              {t("assExport.assignment")}
            </span>
            <span className="grow" />
            <button
              className="assPresetAdd"
              onClick={() => {
                const todos: Record<string, string> = {};
                for (const h of hablantes) todos[h.id] = primero;
                setAsignacion(todos);
                setPresetSinHablante(primero);
              }}
            >
              {t("assExport.applyToAll")}
            </button>
          </div>
          {hablantes.map((h) => (
            <div className="assAssignRow" key={h.id}>
              <span
                className="speakerDot"
                style={{ background: h.color }}
                aria-hidden="true"
              />
              <span className="assAssignName">
                {h.nombre || h.tecla || h.id}
              </span>
              <span className="grow" />
              <select
                value={presetDe(h.id)}
                onChange={(e) =>
                  setAsignacion((a) => ({ ...a, [h.id]: e.target.value }))
                }
              >
                {presets.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.nombre}
                  </option>
                ))}
              </select>
            </div>
          ))}
          <div className="assAssignRow">
            <span className="assAssignDotSin" aria-hidden="true" />
            <span className="assAssignName muted">
              {t("assExport.unassigned")}
            </span>
            <span className="grow" />
            <select
              value={presetSinHablante || primero}
              onChange={(e) => setPresetSinHablante(e.target.value)}
            >
              {presets.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.nombre}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="assFooter">
          <span className="grow" />
          <button className="addFragmentBtn" onClick={onCerrar}>
            {t("assExport.close")}
          </button>
          <button
            className="addFragmentBtn primary"
            onClick={() => onExportar(asignacion, presetSinHablante || primero)}
          >
            {t("assExport.export")}
          </button>
        </div>
      </div>
    </div>
  );
}
