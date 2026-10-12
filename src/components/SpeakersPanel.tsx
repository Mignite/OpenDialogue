import { memo } from "react";
import type { Hablante, PresetAss } from "../types";
import { useLocale } from "../i18n";
import { familiaCss } from "../utils/fuentes";
import { colorHablante } from "../utils/colorHablante";

interface Props {
  hablantes: Hablante[];
  presets: PresetAss[];
  onAgregar: () => void;
  onActualizar: (id: string, campo: keyof Hablante, valor: string) => void;
  onEliminar: (id: string) => void;
  onCommit: () => void;
}

function SpeakersPanel({ hablantes, presets, onAgregar, onActualizar, onEliminar, onCommit }: Props) {
  const { t } = useLocale();

  const xSvg = (
    <svg
      className="icon xs"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.6}
      strokeLinecap="round"
    >
      <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" />
    </svg>
  );

  return (
    <div className="speakersPanel">
      {hablantes.map((h) => {
            // Firma visual: la fila muestra su propio estilo .ass (fuente +
            // subrayado en su color). Solo fuente y subrayado, nunca el color
            // de texto: presets como negro puro serían ilegibles sobre dark.
            const estilo = presets.find((p) => p.id === h.presetId);
            return (
            <div key={h.id} className="speakerRow">
              // El dot es decorativo: muestra el color del preset .ass del
              // hablante (o blanco sin preset). El color se edita en Estilos.
              <span
                className="speakerDot"
                style={{ backgroundColor: colorHablante(h, presets) }}
                title={estilo ? `${estilo.nombre} (${estilo.color})` : t("speakers.presetNone")}
              />
              <input
                className="speakerInput"
                placeholder={t("speakers.namePlaceholder")}
                value={h.nombre}
                onFocus={onCommit}
                onChange={(e) => onActualizar(h.id, "nombre", e.target.value)}
                style={
                  estilo
                    ? {
                        fontFamily: familiaCss(estilo.fontname),
                        // El peso/cursiva del preset se ve aquí: si no, la fila
                        // muestra una fuente que el .ass no va a usar igual.
                        fontWeight: estilo.bold ? 700 : 400,
                        fontStyle: estilo.italic ? "italic" : "normal",
                        boxShadow: `inset 0 -2px 0 ${estilo.color}`,
                      }
                    : undefined
                }
                title={estilo ? estilo.nombre : undefined}
              />
              <input
                className="speakerKeyInput"
                maxLength={1}
                value={h.tecla}
                onFocus={onCommit}
                onChange={(e) => onActualizar(h.id, "tecla", e.target.value)}
                title={t("speakers.keyTitle")}
              />
              <select
                className="speakerPreset"
                value={estilo ? (h.presetId ?? "") : ""}
                onFocus={onCommit}
                onChange={(e) => onActualizar(h.id, "presetId", e.target.value)}
                title={estilo ? estilo.nombre : t("speakers.presetNone")}
              >
                <option value="">{t("speakers.presetNone")}</option>
                {presets.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.nombre}
                  </option>
                ))}
              </select>
              <button className="iconBtnSmall" onClick={() => onEliminar(h.id)} title={t("speakers.deleteTitle")}>
                {xSvg}
              </button>
            </div>
            );
          })}
          {hablantes.length < 9 && (
            <button className="addSpeakerBtn" onClick={onAgregar}>
              {t("speakers.add")}
            </button>
          )}
    </div>
  );
}

export default memo(SpeakersPanel);