# Plan — Consistencia, rendimiento y comodidad (OpenDialogue)

Objetivo del usuario: "que sea una app útil, cómoda y rápida"; prioridad a consistencia,
todo el código en el repo, que compile sólido, más beneficios que pérdidas.

Orden decidido (beneficio/riesgo):

## Fase 1 — Fuentes con `fontdb` (consistencia multiplataforma) — HECHO
- [x] 1.1 `fontdb = "0.24"` en Cargo.toml
- [x] 1.2 Tests Rust de `normalizar_familias` (3): dedup+orden, vacíos, inválidos
- [x] 1.3 `listar_fuentes_sistema` con `fontdb`, sin `#[cfg]` por plataforma
- [x] 1.4 Borrados `familias_fc_list`, `familias_del_registro`, rama PowerShell, `comando_oculto`
- [x] 1.5 Comentarios de `fuentes.ts` actualizados
- [x] 1.6 Gates: 220 vitest + 14 Rust + clippy + fmt + build, todo verde
- [x] 1.7 Bold/Italic (aprobado a mitad de fase): 6 tests ass + 3 tests assPresets

### Medido
- 136 familias canónicas vs 265 de `fc-list` (las 129 de diferencia son
  variantes de peso, que en .ass van en el campo Bold, no en el Fontname).
- fontdb agrupa pesos bajo la familia base: "Cantarell Light"/"Cantarell Bold"
  reportan `Cantarell`. Carga en frío ~140 ms.
- La app ya no shellea ningún proceso hijo (ni powershell, ni reg, ni fc-list).

## Fase 2 — Rendimiento (bajo riesgo, alto impacto)
- [ ] 2.1 `useMemo` en `trackRows` (App.tsx) — O(n·S) por render
- [ ] 2.2 `useMemo` en `matchingCaptions` (App.tsx)
- [ ] 2.3 `Set` en vez de `Array.includes()` en mutaciones (useCaptions.ts, App.tsx)
- [ ] 2.4 Cachear color del canvas por cubos (48k literals/frame)
- [ ] 2.5 Sacar lectura de `clientWidth` del render → ResizeObserver/ref (layout thrash)

## Fase 3 — Estabilidad
- [ ] 3.1 Autosave debounced + recovery file al arrancar (hoy NO existe)
- [ ] 3.2 Limpiar los 5 `setTimeout` crudos → un solo mecanismo `notify`
- [ ] 3.3 `clearTimeout` en debounce de estilos (pierde último cambio al cerrar <400ms)
- [ ] 3.4 Guardar `playheadPendienteRef` solo si el video existe (contamina el flujo "seguir sin video")

## Fase 4 — Comodidad
- [ ] 4.1 Portapapeles real del SO (Ctrl+C/V cruzan con otras apps)
- [ ] 4.2 Editor numérico inicio/fin del caption + Shift+←/→ frame-step
- [ ] 4.3 Resolver video por ruta relativa al `.json` (portabilidad entre máquinas/OS)

## Fase 5 — Pulido
- [ ] 5.1 Traducir `ALINEACIONES` + 3 strings de error hardcodeados
- [ ] 5.2 Actualizar README (conteo de tests, estructura)

## Reglas
- TDD en cada cambio de comportamiento: test falla primero, se ve fallar, código mínimo.
- Gates tras cada fase: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`,
  `npm run build`, `npm test`.
- Un cambio a la vez; nada de "ya que estás" mezclado.