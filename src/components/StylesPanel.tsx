import { memo, useEffect, useRef, useState } from "react";
import type { PresetAss } from "../types";
import { useLocale } from "../i18n";
import { nuevoPreset, nombreLibre } from "../utils/assPresets";
import { fuentesDisponibles, fuenteResuelve, familiaCss } from "../utils/fuentes";

interface Props {
  presets: PresetAss[];
  panelAbierto: boolean;
  onTogglePanel: () => void;
  onGuardar: (presets: PresetAss[]) => void;
}

const ALINEACIONES: { valor: number; etiqueta: string }[] = [
  { valor: 1, etiqueta: "1 Abajo izq." },
  { valor: 2, etiqueta: "2 Abajo centro" },
  { valor: 3, etiqueta: "3 Abajo der." },
  { valor: 4, etiqueta: "4 Medio izq." },
  { valor: 5, etiqueta: "5 Medio centro" },
  { valor: 6, etiqueta: "6 Medio der." },
  { valor: 7, etiqueta: "7 Arriba izq." },
  { valor: 8, etiqueta: "8 Arriba centro" },
  { valor: 9, etiqueta: "9 Arriba der." },
];

const CAMPOS_NUMERO: (keyof PresetAss)[] = [
  "fontsize",
  "outline",
  "shadow",
  "marginL",
  "marginR",
  "marginV",
];

/** Panel de estilos .ass de primera clase: vive en la columna derecha y se
 *  puede editar en cualquier momento, sin subtítulos ni export de por medio.
 *  Todo cambio se auto-guarda (nivel usuario, fuera del undo del proyecto):
 *  no hay borrador ni confirmación de descarte posibles. */
function StylesPanel({ presets, panelAbierto, onTogglePanel, onGuardar }: Props) {
  const { t } = useLocale();
  const [seleccionId, setSeleccionId] = useState<string>(presets[0]?.id ?? "");
  const [fuentes, setFuentes] = useState<string[]>([]);
  const [fuentesCargadas, setFuentesCargadas] = useState(false);
  const [listaFuentesAbierta, setListaFuentesAbierta] = useState(false);
  const inputFuenteRef = useRef<HTMLInputElement>(null);

  // La promesa de fuentes está cacheada por sesión (fuentes.ts).
  useEffect(() => {
    let vivo = true;
    fuentesDisponibles().then((f) => {
      if (!vivo) return;
      setFuentes(f);
      setFuentesCargadas(true);
    });
    return () => {
      vivo = false;
    };
  }, []);

  // Derivado de props, nunca stale: si el seleccionado desaparece (import
  // externo, borrado en otra superficie) se cae al primero sin romper.
  const sel = presets.find((p) => p.id === seleccionId) ?? presets[0] ?? null;

  const editar = <C extends keyof PresetAss>(
    campo: C,
    valor: PresetAss[C],
  ) => {
    if (!sel) return;
    const actualizado = { ...sel, [campo]: valor };
    onGuardar(presets.map((p) => (p.id === sel.id ? actualizado : p)));
  };

  const agregarPreset = () => {
    // Parte del preset actual: casi siempre querés tweaked lo que ya te gusta,
    // no volver de cero. El nombre se renumera solo.
    const base = nombreLibre(sel?.nombre ?? "Preset", presets.map((p) => p.nombre));
    const nuevo = nuevoPreset({ ...(sel ?? {}), nombre: base });
    onGuardar([...presets, nuevo]);
    setSeleccionId(nuevo.id);
  };

  const borrar = () => {
    if (!sel || presets.length <= 1) return;
    const resto = presets.filter((p) => p.id !== sel.id);
    onGuardar(resto);
    setSeleccionId(resto[0].id);
  };

  // Con el campo vacio se muestran las primeras; con texto, las que lo contienen.
  const consulta = (sel?.fontname ?? "").trim().toLowerCase();
  const filtradas = fuentes
    .filter((f) => f.toLowerCase().includes(consulta))
    .slice(0, 40);

  const chevronSvg = (
    <svg
      className={"icon xs chevron" + (panelAbierto ? " up" : "")}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
    >
      <path d="M3 6l5 5 5-5" />
    </svg>
  );

  return (
    <div className="speakersAccordion">
      <button className="speakersAccordionHeader" onClick={onTogglePanel}>
        <span>{t("styles.title", { count: presets.length })}</span>
        {chevronSvg}
      </button>
      {panelAbierto && (
        <div className="stylesPanel">
          <div className="assPresetList">
            {presets.map((p) => (
              <button
                key={p.id}
                className={"assPresetItem" + (sel && p.id === sel.id ? " activo" : "")}
                onClick={() => setSeleccionId(p.id)}
              >
                <span
                  className="assPresetSwatch"
                  style={{ background: p.color }}
                  aria-hidden="true"
                />
                {p.nombre}
              </button>
            ))}
            <div className="assPresetActions">
              <button
                className="assPresetAdd"
                onClick={agregarPreset}
                title={t("assExport.newPreset")}
              >
                <span className="assPresetAddPlus" aria-hidden="true">
                  +
                </span>
                {t("assExport.newPreset")}
              </button>
              <button
                className="assPresetDel"
                onClick={borrar}
                disabled={presets.length <= 1}
                title={t("assExport.delete")}
              >
                {t("assExport.delete")}
              </button>
            </div>
          </div>

          {sel && (
            <div className="assFields">
              <div className="assField">
                <label>{t("assExport.field_nombre")}</label>
                <input
                  value={sel.nombre}
                  onChange={(e) => editar("nombre", e.target.value)}
                />
              </div>
              <div className="assField">
                <label>{t("assExport.field_fontname")}</label>
                <div className="assFontPicker">
                  <input
                    ref={inputFuenteRef}
                    className="assFontInput"
                    value={sel.fontname}
                    style={{ fontFamily: familiaCss(sel.fontname) }}
                    placeholder={t("assExport.fontPlaceholder")}
                    onFocus={() => setListaFuentesAbierta(true)}
                    onBlur={() =>
                      window.setTimeout(() => setListaFuentesAbierta(false), 150)
                    }
                    onChange={(e) => editar("fontname", e.target.value)}
                  />
                  <button
                    type="button"
                    className={
                      "assFontCaret" + (listaFuentesAbierta ? " abierto" : "")
                    }
                    title={t("assExport.fontPicker")}
                    aria-label={t("assExport.fontPicker")}
                    aria-expanded={listaFuentesAbierta}
                    // preventDefault para que el click no robe el foco: si lo
                    //-robara, el onBlur del input cerraria la lista al instante.
                    onMouseDown={(e) => e.preventDefault()}
                    onClick={() => {
                      if (listaFuentesAbierta) {
                        setListaFuentesAbierta(false);
                      } else {
                        inputFuenteRef.current?.focus();
                      }
                    }}
                  >
                    <svg
                      className="chevron"
                      viewBox="0 0 16 16"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth={1.8}
                      strokeLinecap="round"
                    >
                      <path d="M3 6l5 5 5-5" />
                    </svg>
                  </button>
                </div>
                {listaFuentesAbierta && (
                  <div className="assFontList">
                    {!fuentesCargadas ? (
                      <div className="assFontItem muted">
                        {t("assExport.fontsLoading")}
                      </div>
                    ) : (
                      <>
                        {filtradas.length === 0 && (
                          <div className="assFontItem muted">
                            {t("assExport.fontsEmpty")}
                          </div>
                        )}
                        {filtradas.map((f) => (
                          <button
                            key={f}
                            className="assFontItem"
                            style={{ fontFamily: familiaCss(f) }}
                            onMouseDown={(e) => e.preventDefault()}
                            onClick={() => {
                              editar("fontname", f);
                              setListaFuentesAbierta(false);
                            }}
                          >
                            {f}
                          </button>
                        ))}
                        {sel.fontname.trim() !== "" &&
                          !fuenteResuelve(sel.fontname) && (
                            <div className="assFontItem muted">
                              {t("assExport.fontNotInstalled")}
                            </div>
                          )}
                      </>
                    )}
                  </div>
                )}
              </div>
              {CAMPOS_NUMERO.map((campo) => (
                <div className="assField" key={campo}>
                  <label>{t(`assExport.field_${campo}`)}</label>
                  <input
                    type="number"
                    value={sel[campo] as number}
                    onChange={(e) =>
                      editar(campo, Number(e.target.value) || 0)
                    }
                  />
                </div>
              ))}
              <div className="assField">
                <label>{t("assExport.field_alignment")}</label>
                <select
                  value={sel.alignment}
                  onChange={(e) => editar("alignment", Number(e.target.value))}
                >
                  {ALINEACIONES.map((a) => (
                    <option key={a.valor} value={a.valor}>
                      {a.etiqueta}
                    </option>
                  ))}
                </select>
              </div>
              <div className="assField">
                <label>{t("assExport.field_color")}</label>
                <input
                  type="color"
                  value={sel.color}
                  onChange={(e) => editar("color", e.target.value)}
                />
              </div>
              <div className="assField">
                <label>{t("assExport.field_outlineColor")}</label>
                <input
                  type="color"
                  value={sel.outlineColor}
                  onChange={(e) => editar("outlineColor", e.target.value)}
                />
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default memo(StylesPanel);
