import { memo, useEffect, useRef, useState } from "react";
import type { PresetAss } from "../types";
import { useLocale } from "../i18n";
import { nuevoPreset, nombreLibre } from "../utils/assPresets";
import { fuentesDisponibles, fuenteResuelve, familiaCss } from "../utils/fuentes";
import StylePreview from "./StylePreview";

interface Props {
  presets: PresetAss[];
  panelAbierto: boolean;
  textoInicial: string;
  onTogglePanel: () => void;
  onGuardar: (presets: PresetAss[]) => void;
}

// Orden visual de arriba hacia abajo: la fila superior es alineación alta
// (7-9), la inferior es baja (1-3), como libass piensa la Alignment.
const ORDEN_ALINEACION: number[] = [7, 8, 9, 4, 5, 6, 1, 2, 3];

// Trazos de las flechas (el centro, 5, no lleva: no apunta a ninguna esquina).
const FLECHAS_ALINEACION: Record<number, string> = {
  7: "M12.5 12.5L4 4M4 9V4h5",
  8: "M8 13V3M4.5 6.5L8 3l3.5 3.5",
  9: "M3.5 12.5L12 4M7 4h5v5",
  4: "M13 8H3M6.5 4.5L3 8l3.5 3.5",
  6: "M3 8h10M9.5 4.5L13 8l-3.5 3.5",
  1: "M12.5 3.5L4 12M9 12H4V7",
  2: "M8 3v10M4.5 9.5L8 13l3.5-3.5",
  3: "M3.5 3.5L12 12M12 7v5H7",
};

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
function StylesPanel({ presets, panelAbierto, textoInicial, onTogglePanel, onGuardar }: Props) {
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

  // Glifo de la matriz de alineación: SVG inline currentColor, nunca
  // carácter Unicode (las flechas tipográficas se renderizan distinto según
  // la fuente y romperían la grilla).
  const glifoAlineacion = (valor: number) =>
    valor === 5 ? (
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <circle cx="8" cy="8" r="2.6" fill="currentColor" />
      </svg>
    ) : (
      <svg
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        strokeWidth={1.8}
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d={FLECHAS_ALINEACION[valor]} />
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
                    style={{
                      fontFamily: familiaCss(sel.fontname),
                      fontWeight: sel.bold ? 700 : 400,
                      fontStyle: sel.italic ? "italic" : "normal",
                    }}
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
              {/* Peso y cursiva: en .ass son los campos Bold/Italic del
                  Style:, no parte del nombre de la fuente. Por eso el selector
                  de arriba lista solo familias canónicas ("Cantarell") y acá
                  se elige el peso ("Cantarell" + negrita = Cantarell Bold). */}
              <div className="assField">
                <label>{t("assExport.field_weight")}</label>
                <div className="assToggleRow">
                  <button
                    type="button"
                    className={
                      "assToggle" + (sel.bold === true ? " activo" : "")
                    }
                    aria-pressed={sel.bold === true}
                    onClick={() => editar("bold", sel.bold !== true)}
                  >
                    <span className="assToggleGlyph">B</span>
                    {t("assExport.bold")}
                  </button>
                  <button
                    type="button"
                    className={
                      "assToggle" + (sel.italic === true ? " activo" : "")
                    }
                    aria-pressed={sel.italic === true}
                    onClick={() => editar("italic", sel.italic !== true)}
                  >
                    <span className="assToggleGlyph italics">I</span>
                    {t("assExport.italic")}
                  </button>
                </div>
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
                <div
                  className="assAlignGrid"
                  role="group"
                  aria-label={t("assExport.field_alignment")}
                >
                  {ORDEN_ALINEACION.map((valor) => (
                    <button
                      key={valor}
                      type="button"
                      className={
                        "assAlignBtn" +
                        (sel.alignment === valor ? " activo" : "")
                      }
                      aria-pressed={sel.alignment === valor}
                      aria-label={t(`assExport.align_${valor}`)}
                      // preventDefault para que el click no robe el foco: sin
                      // esto el botón quedaría focalizado y la tecla siguiente
                      // la leería el handler global (p. ej. → = seek +5s).
                      // Mismo patrón que el caret del selector de fuentes.
                      onMouseDown={(e) => e.preventDefault()}
                      onClick={() => editar("alignment", valor)}
                    >
                      {glifoAlineacion(valor)}
                    </button>
                  ))}
                </div>
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
          {sel && <StylePreview preset={sel} textoInicial={textoInicial} />}
        </div>
      )}
    </div>
  );
}

export default memo(StylesPanel);
