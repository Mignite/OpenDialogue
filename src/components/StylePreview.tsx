import { memo, useEffect, useRef, useState } from "react";
import type { PresetAss } from "../types";
import { useLocale } from "../i18n";
import { estiloCajaPreview, estiloLineaPreview } from "../utils/assPreview";

interface Props {
  preset: PresetAss;
  textoInicial: string;
}

/** Preview en vivo del preset: un stage 16:9 negro que renderiza el texto de
 *  ejemplo con `estiloLineaPreview` / `estiloCajaPreview` — el MISMO código
 *  que el overlay sobre el video, no una copia. La escala es
 *  `min(w/1920, h/1080)` porque libass escala por PlayResY (manda el menor de
 *  los dos ejes). La medida vive en un ref + tick de re-render (patrón del
 *  video en App.tsx), no en un useState que re-renderice en cada frame. */
function StylePreview({ preset, textoInicial }: Props) {
  const { t } = useLocale();
  const [texto, setTexto] = useState(textoInicial);
  const stageRef = useRef<HTMLDivElement>(null);
  const cajaRef = useRef({ w: 0, h: 0 });
  const [, setTick] = useState(0);

  useEffect(() => {
    const el = stageRef.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const medir = () => {
      cajaRef.current = { w: el.clientWidth, h: el.clientHeight };
      setTick((n) => n + 1);
    };
    medir();
    const ro = new ResizeObserver(medir);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const { w, h } = cajaRef.current;
  const medido = w > 0 && h > 0;
  // Antes del primer ResizeObserver la caja es 0x0. Escala 0 da fontSize 0 y
  // márgenes 0 — el texto desaparece. Se usa 1 como escala nominal hasta que
  // haya medida real; el observer dispara un re-render en cuanto la hay.
  const escala = medido ? Math.min(w / 1920, h / 1080) : 1;

  return (
    <div className="stylePreviewBlock">
      <div className="stylePreviewLabel">{t("styles.previewLabel")}</div>
      <input
        className="stylePreviewInput"
        value={texto}
        placeholder={t("styles.previewTextPlaceholder")}
        onChange={(e) => setTexto(e.target.value)}
      />
      <div
        ref={stageRef}
        className="stylePreviewStage"
        style={medido
          ? estiloCajaPreview(preset, { left: 0, top: 0, width: w, height: h }, escala)
          : undefined}
      >
        <span style={medido ? estiloLineaPreview(preset, escala) : undefined}>
          {texto}
        </span>
      </div>
    </div>
  );
}

export default memo(StylePreview);
