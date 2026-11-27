# Rediseño visual de la UI — Plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Llevar Inicio y el editor al lenguaje visual del mockup `OpenDialogue UI.html` (denso, oscuro, un solo acento), con identidad de color por hablante, timeline con waveform, y Ajustes con idioma y atajos configurables, sin perder ni un comportamiento del editor actual.

**Architecture:** `tema.rs` pasa a ser la única fuente de color, radio, espacio y tamaño; `ui/kit.rs` concentra los widgets; cada vista recibe `&Textos` y `&MapaAtajos` y devuelve mensajes, como ya hace hoy. El estado (proyecto, historial, selección) no se toca: esta obra es de presentación y de dos módulos nuevos de datos (`preferencias.rs`, `textos.rs`).

**Tech Stack:** Rust 2024, egui/eframe `=0.33.0`, egui-phosphor 0.11, egui_kittest 0.33 (dev), symphonia (rod-player), Inter + JetBrains Mono embebidas.

**Spec:** `docs/superpowers/specs/2026-11-24-editor-ui-redesign-design.md` (leerla junto a este plan; los valores exactos viven ahí).

## Global Constraints

- egui/eframe `=0.33.0`. `CornerRadius`, nunca `Rounding`.
- Paleta, radios, espacios y alturas: solo los de la spec §2.1, y solo en `tema.rs`. Ningún `Color32::from_rgb`, `.size(…)` ni `corner_radius` literal en el resto de `ui/`.
- `#3B8EFF` se usa **solo** en foco, selección, playhead y marca activa. El botón primario es `#2766DB`.
- Ningún texto por debajo de 11 px. Mono solo en cifras, atajos y códigos técnicos: nunca en prosa.
- Todo par texto/fondo ≥ 4.5:1, verificado por test.
- Dependencias permitidas: `egui-phosphor = "0.11"`; dev: `egui_kittest = "0.33"` (wgpu, snapshot, eframe), `image`, `ttf-parser`; `symphonia` solo en `rod-player`.
- `rod-core` no depende de egui: su API devuelve `&'static str` y `bool`, nunca tipos de UI.
- El paquete se llama `rod-core`; nunca `core` (rompe `core::time`).
- Sin `unsafe`. Sin GStreamer.
- Identificadores y comentarios en español; toda cadena visible en `textos.rs`.
- Idioma por defecto: inglés. Español disponible en Ajustes.
- `%APPDATA%\RustyOpenDialogue` no se mueve.
- Repintado solo al reproducir, animar, durante un seek, o con cambios sin guardar.
- Cada tarea cierra con `cargo fmt`, `cargo clippy -p rod-app -D warnings`, `cargo test` y commit.

## Review Focus

1. **Escribir en un campo de texto** con atajos de tecla desnuda vigentes (`Space`, `E`, `A`, `C`, `1`–`9`): debe insertar caracteres y no pausar, dividir ni asignar hablante. (Tareas 6, 13, 14.)
2. **Proyectos con más de 12 hablantes, o con `Hablante.color` blanco, inválido o repetido**: deben recibir colores distintos de la paleta, sin pánico ni texto invisible sobre el fondo. (Tareas 10, 15.)
3. **Nombres de hablante y textos muy largos a 1100 px**: elipsis con tooltip, sin empujar paneles, sin cortar la barra de transporte ni desbordar la cabecera. (Tareas 12, 20.)
4. **Video sin pista de audio, o con códec que symphonia no decodifica**: la app abre igual, el timeline funciona y la pista de audio queda vacía, sin diálogo de error. (Tarea 16.)
5. **`preferencias.json` corrupto, de otra versión o de solo lectura**: arranca con valores de fábrica, sigue funcionando y el proyecto se guarda igual. (Tareas 6, 11.)

## Fixture compartida

Todas las pruebas y el ejemplo de capturas usan la misma. Crear
`crates/app/tests/comun/mod.rs` (los `tests/*.rs` la importan con
`mod comun;` y `use comun::*;`) y `crates/app/src/fixture.rs` para que el
ejemplo también pueda usarla:

```rust
/// 7 hablantes, 543 captions de 0.04 s espaciados 0.06 s (~19.6 s).
/// El video es el clip real del repo, así que mpv/libass y la waveform
/// tienen datos de verdad; no depende de archivos del usuario.
pub fn proyecto_543() -> rod_core::Proyecto {
    let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/clip-15s-2audio.mp4");
    let mut proyecto = rod_core::Proyecto::nuevo(ruta.to_string_lossy().into_owned());
    for i in 0..7 {
        proyecto.hablantes.push(rod_core::Hablante {
            id: format!("h{i}"),
            nombre: format!("Hablante {i}"),
            tecla: String::new(),
            color: String::new(),
            preset_id: None,
        });
    }
    for n in 0..543 {
        let inicio = n as f64 * 0.06;
        proyecto.captions.push(rod_core::Caption {
            id: format!("c{n}"),
            inicio,
            fin: inicio + 0.04,
            texto: format!("Línea {n} del hablante {}", n % 7),
            hablante_id: Some(format!("h{}", n % 7)),
        });
    }
    proyecto
}

pub fn editor_543() -> rod_app::editor_app::EditorApp {
    let dir = std::env::temp_dir().join(format!("rod-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    rod_app::editor_app::EditorApp::desde_proyecto_en(
        proyecto_543(), None, 0.0, &dir,
    ).expect("abre el clip de prueba")
}
```

Los campos son los reales de `rod_core` (verificados en `model.rs`):
`Hablante { id, nombre, tecla: String, color, preset_id }` y
`Caption { id, inicio, fin, texto, hablante_id }`. En los snippets de las
tareas siguientes, `editor_de_prueba()` y `estado_con_tres()` son atajos para
`comun::editor_543()` y un `EstadoInicio` con tres `Reciente` de la fixture.
Si mpv no puede abrir sin ventana en el harness, el test del editor se marca
`#[ignore = "mpv necesita ventana"]`, se anota en `BITACORA.md` y se prueba
cada panel por separado con la misma fixture.

---

### Task 1: Arnés de capturas y estado «antes»

**Files:**
- Create: `crates/app/tests/kittest.rs`, `crates/app/examples/capturas.rs`
- Modify: `crates/app/Cargo.toml` (dev-dependencies)

**Interfaces:**
- Produces: `fn capturar(nombre: &str, ancho: f32, alto: f32, ui: impl FnMut(&mut egui::Ui))` en el ejemplo, que escribe `docs/checklists/rediseno/<nombre>.png`.
- Consumes: `inicio::{EstadoInicio, nuevo}`, `recientes::Reciente`.

- [ ] **Step 1: Test que prueba que kittest renderiza en esta máquina**

```rust
// crates/app/tests/kittest.rs
#[test]
fn kittest_renderiza_un_rectangulo() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(64.0, 64.0))
        .build_ui(|ui| {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, egui::Color32::RED);
        });
    harness.run();
    let imagen = harness.render().expect("renderiza");
    // render() devuelve píxeles, no points: con pixels_per_point = 1 coinciden.
    assert_eq!((imagen.width(), imagen.height()), (64, 64));
    let rojos = imagen.pixels().filter(|p| p[0] > 200).count();
    assert!(rojos > 64 * 64 / 2, "el rectángulo no se pintó: {rojos}");
}
```

- [ ] **Step 2: Ejecutarlo y ver que falla por la dependencia**

Run: `cargo test -p rod-app --test kittest`
Expected: FAIL, `unresolved import egui_kittest`.

- [ ] **Step 3: Añadir las dev-dependencies**

En `crates/app/Cargo.toml`, sección `[dev-dependencies]`:

```toml
egui_kittest = { version = "0.33", features = ["wgpu", "snapshot", "eframe"] }
image = "0.25"
```

- [ ] **Step 4: Ejecutar el test**

Run: `cargo test -p rod-app --test kittest`
Expected: PASS. Si falla por falta de adaptador wgpu, dejar el test con `#[ignore = "sin adaptador wgpu en esta máquina"]` y anotarlo en `BITACORA.md`; el resto del plan sigue.

- [ ] **Step 5: Escribir el ejemplo de capturas con Inicio**

`crates/app/examples/capturas.rs`: crea `docs/checklists/rediseno/` si falta y genera `antes-inicio-vacio.png` (un `EstadoInicio` con `recientes: vec![]`) y `antes-inicio-3.png` (tres `Reciente` de ejemplo: `presa.cdp` 19 líneas/3 hablantes, `RobloxTerror.cdp` 543/7, `proyecto.cdp` 543/7) a 1440×900. El builder del harness se configura con `.with_size(vec2(1440.0, 900.0)).with_pixels_per_point(1.0)`. La firma actual es `ui::inicio::mostrar(ui: &mut Ui, estado: &mut EstadoInicio) -> Option<MensajeInicio>`: el ejemplo ignora el mensaje devuelto. La tarjeta «antes» no tiene miniatura ni fecha relativa traducible todavía: esas son las capturas de partida.

- [ ] **Step 6: Generar y mirar las capturas**

Run: `cargo run -p rod-app --example capturas -- antes`
Expected: dos PNG en `docs/checklists/rediseno/`. Abrirlos: tienen que mostrar la pantalla de Inicio actual (marca `RUSTYDIALOGUE`, tarjetas sin miniatura). Son el «antes» del proyecto.

- [ ] **Step 7: Captura real del editor «antes»**

`Harness::render()` dibuja los shapes de egui pero **no ejecuta el
`PaintCallback` de mpv**, así que el monitor sale vacío en las capturas
headless. Con el PC disponible: `cargo run -p rod-app`, abrir
`testdata/clip-15s-2audio.mp4`, dejar la ventana a 1440×900 y capturarla con
`docs/checklists/rediseno/capturar.ps1` (`Add-Type -AssemblyName System.Drawing`
+ `CopyFromScreen`). Guardar `antes-editor-1440.png`. Sustituye al antiguo
`redesign-antes-editor.png`, que mostraba el Inicio y fue descartado.

- [ ] **Step 8: Commit**

```bash
git add crates/app/tests/kittest.rs crates/app/examples/capturas.rs crates/app/Cargo.toml Cargo.lock docs/checklists/rediseno
git commit -m "test: arnes de capturas headless, antes de Inicio y del editor"
```

---

### Task 2: Fuentes y tipografía

**Files:**
- Create: `crates/app/assets/fonts/` (5 TTF + 2 licencias), `crates/app/src/ui/tipografia.rs`
- Modify: `crates/app/src/ui/mod.rs`

**Interfaces:**
- Produces: `pub enum Peso { Regular, Medium, SemiBold, Mono }`; `pub fn familia(peso: Peso) -> egui::FontFamily`; `pub fn instalar(ctx: &egui::Context)`.

- [ ] **Step 1: Descargar y subconjuntar las fuentes**

```bash
mkdir -p crates/app/assets/fonts
# Inter desde github.com/rsms/inter (release) y JetBrains Mono desde github.com/JetBrains/JetBrainsMono
pip install fonttools
pyftsubset Inter-Regular.ttf --output-file=crates/app/assets/fonts/Inter-Regular.ttf \
  --unicodes="U+0020-00FF,U+0100-017F,U+2018-201D,U+2026,U+2190-2193,U+2212"
# repetir para Inter-Medium, Inter-SemiBold, JetBrainsMono-Regular, JetBrainsMono-Medium
```

Copiar `OFL.txt` de cada familia a `OFL-Inter.txt` y `OFL-JetBrainsMono.txt`.

- [ ] **Step 2: Test de que las familias nombradas existen y no entran en pánico**

```rust
// inline en tipografia.rs
#[test]
fn las_familias_nombradas_se_registran_y_miden_distinto() {
    let ctx = egui::Context::default();
    instalar(&ctx);
    let medir = |peso| ctx.fonts(|f| {
        f.layout_no_wrap("Hablante".into(), egui::FontId::new(13.0, familia(peso)), egui::Color32::WHITE)
            .size().x
    });
    assert!(medir(Peso::Regular) > 0.0);
    assert_ne!(medir(Peso::Medium), medir(Peso::Regular), "Medium no se cargó");
    assert_ne!(medir(Peso::Mono), medir(Peso::Regular), "la mono no se cargó");
    let _ = medir(Peso::SemiBold);
}
```

- [ ] **Step 3: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app tipografia`
Expected: FAIL, `no existe la función instalar`.

- [ ] **Step 4: Implementar `instalar`**

`FontDefinitions::default()`, insertar las familias: `Proportional` = `[Inter-Regular, NotoEmoji-Regular]` (NotoEmoji sale de `FontDefinitions::default()`, conservarlo como reserva), `Name("inter-medium")`, `Name("inter-semibold")`, `Monospace` = `[JetBrainsMono-Regular]`. Cargar con `include_bytes!` y `FontData::from_static`. `ctx.set_fonts(definiciones)`.

Los iconos **no** se cargan aquí. `egui-phosphor` entra en la tarea 4 y se registra con su API pública, no a mano:

```rust
egui_phosphor::add_to_fonts(&mut definiciones, egui_phosphor::Variant::Regular);
```

Esa función mete la fuente bajo la clave `"phosphor"` y la inserta en el índice 1 de `FontFamily::Proportional` (verificado en la fuente de egui-phosphor 0.11). Por eso hay que llamarla **después** de fijar Inter como primer elemento de `Proportional`: los glifos de Phosphor caen como reserva automática en cualquier texto con familia proporcional.

- [ ] **Step 5: Ejecutar el test y comprobar que la app sigue abriendo**

Run: `cargo test -p rod-app tipografia`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/app/assets/fonts crates/app/src/ui/tipografia.rs crates/app/src/ui/mod.rs
git commit -m "feat: Inter y JetBrains Mono embebidas con familias nombradas"
```

---

### Task 3: Tema completo

**Files:**
- Rewrite: `crates/app/src/ui/tema.rs`
- Test: inline en el mismo archivo

**Interfaces:**
- Produces: constantes de la spec §2.1 (`FONDO`, `RIEL`, `PANEL`, `CONTROL`, `ELEVADO`, `ACTIVO`, `BORDE`, `BORDE_CONTROL`, `BORDE_HOVER`, `TEXTO`, `TEXTO_MENU`, `TEXTO_2`, `TEXTO_3`, `ACENTO`, `PRIMARIO`, `PRIMARIO_HOVER`, `PELIGRO`, `RADIO_CONTROL`, `RADIO_SUPERFICIE`, `ESP_2`…`ESP_72`, `ALTO_CHIP`, `ALTO_CONTROL`, `ALTO_NAV`, `ALTO_BOTON`, `ALTO_CABECERA`, `ANCHO_BOTON_VENTANA`, `ALTO_FILA_SUB`); `pub enum Estilo { Titulo, Subtitulo, Cuerpo, CuerpoFuerte, Base, Pequeno, Etiqueta, Mono, Timecode, TimecodeGrande }`; `pub fn fuente(Estilo) -> egui::FontId`; `pub fn color(hex: &str) -> egui::Color32`; `pub fn aplicar(ctx: &egui::Context)`; `pub fn panel(fill) -> egui::Frame`; `pub fn etiqueta(ui, texto)`.

- [ ] **Step 1: Test de contraste y tamaños**

```rust
fn contraste(a: Color32, b: Color32) -> f32 { /* luminancia relativa, fórmula WCAG */ }

#[test]
fn todo_texto_cumple_aa() {
    for (texto, fondo) in [
        (TEXTO, FONDO), (TEXTO_2, FONDO), (TEXTO_3, FONDO),
        (TEXTO, PANEL), (TEXTO_2, PANEL), (TEXTO_3, PANEL),
        (TEXTO_2, CONTROL), (Color32::WHITE, PRIMARIO), (TEXTO_MENU, FONDO),
    ] {
        let c = contraste(texto, fondo);
        assert!(c >= 4.5, "{texto:?} sobre {fondo:?} da {c:.2}");
    }
}

#[test]
fn ningun_estilo_baja_de_once() {
    for estilo in Estilo::TODAS {
        assert!(fuente(*estilo).size >= 11.0, "{estilo:?}");
    }
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app tema`
Expected: FAIL por constantes ausentes.

- [ ] **Step 3: Escribir los tokens y `aplicar`**

Valores literales de la spec §2.1. `aplicar` llama a `tipografia::instalar`, fija `Style` (`visuals` con `panel_fill`, `window_fill`, `extreme_bg_color`, `selection.bg_fill = ACENTO`, `widgets.*.corner_radius`, `animation_time = 0.14`, `interaction.tooltip_delay = 0.4`), inserta los diez `TextStyle` y fija `spacing.item_spacing`, `button_padding` y `interact_size` con los valores del tema. `main.rs` cambia su `set_style(tema::estilo())` por `tema::aplicar(&cc.egui_ctx)`.

- [ ] **Step 4: Ejecutar los tests**

Run: `cargo test -p rod-app tema`
Expected: PASS. Si algún par no llega a 4.5, el color está mal copiado: corregir contra la spec, no bajar el umbral.

- [ ] **Step 5: Capturas del cambio**

Run: `cargo run -p rod-app --example capturas -- fase1`
Expected: `fase1-inicio-*.png` con la nueva tipografía y paleta, misma distribución.

- [ ] **Step 6: Commit**

```bash
git add crates/app/src/ui/tema.rs crates/app/src/main.rs docs/checklists/rediseno
git commit -m "feat: tema con los tokens del mockup y contraste AA verificado"
```

---

### Task 4: Kit de componentes

**Files:**
- Create: `crates/app/src/ui/kit.rs`
- Modify: `crates/app/src/ui/mod.rs`, `crates/app/src/ui/iconos.rs` (reescrito)

**Interfaces:**
- Consumes: `tema::{Estilo, fuente, color, …}`.
- Produces: `pub enum Icono { Play, Pausa, Inicio, Fin, CuadroAnterior, CuadroSiguiente, Video, Carpeta, Mas, Menos, Cerrar, Minimizar, Maximizar, Ajustes, Recientes, Inicio2, Buscar, Papelera, Deshacer, Rehacer, Volumen, Bucle, Candado, Ojo, Mute, Solo, ChevronAbajo, Punto, Reloj, Subir, Flecha, Descargar, Guardar, Rayo }` con `pub fn glifo(icono: Icono) -> &'static str`; `pub fn icono(ui, Icono) -> egui::Response` (sin caja); `pub fn icon_button(ui, Icono, activo: bool, tooltip: &str, atajo: Option<&str>) -> egui::Response`; `pub fn boton_primario(ui, &str, Icono) -> egui::Response`; `pub fn boton_secundario(ui, &str, Icono) -> egui::Response`; `pub fn panel_header(ui, &str) -> egui::Response`; `pub fn tab_bar(ui, &[(Panel, &str, Option<usize>)], Panel) -> Option<Panel>`; `pub fn punto(ui, Color32, f32)`; `pub fn chip_hablante(ui, &str, Color32) -> egui::Response`; `pub fn campo_tiempo(ui, &mut String, bool) -> egui::Response`; `pub fn campo_texto(ui, &mut String, f32) -> egui::Response`; `pub fn recuadro_punteado(ui, f32, &str)`; `pub fn tarjeta<R>(ui, impl FnOnce(&mut Ui) -> R) -> (egui::Response, R)`; `pub fn separador(ui)`.

- [ ] **Step 1: Añadir las dependencias del kit**

En `crates/app/Cargo.toml`: `egui-phosphor = "0.11"` en `[dependencies]` y `ttf-parser = "0.25"` en `[dev-dependencies]`. La versión 0.11 depende de egui 0.33, la misma del workspace.

- [ ] **Step 2: Test de que no hay tofu en los iconos**

```rust
#[test]
fn todos_los_iconos_existen_en_la_fuente() {
    let bytes = egui_phosphor::Variant::Regular.font_bytes();
    let cara = ttf_parser::Face::parse(bytes, 0).expect("fuente válida");
    for icono in Icono::TODAS {
        let texto = glifo(*icono);
        let mut chars = texto.chars();
        let c = chars.next().expect("un glifo");
        assert!(chars.next().is_none(), "{icono:?} debe ser un solo glifo");
        assert!(cara.glyph_index(c).is_some(), "{icono:?} no está en la fuente");
    }
}
```

`Variant::font_bytes() -> &'static [u8]` es API pública de egui-phosphor 0.11 (confirmada en su fuente). `glifo` devuelve constantes de `egui_phosphor::regular::*`; nunca un carácter unicode escrito a mano.

- [ ] **Step 3: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app iconos`
Expected: FAIL, `Icono` no existe.

- [ ] **Step 4: Implementar `Icono` y los widgets**

`glifo` devuelve las constantes de `egui_phosphor::regular::*` que correspondan a cada variante. Todos los widgets toman medidas y colores de `tema`; ninguno escribe un literal. `icon_button` dibuja hover (`ELEVADO`), activo (`CONTROL`) y el anillo de foco de 2 px; su tooltip usa `.on_hover_ui` con texto + atajo en mono. `campo_tiempo` fija el ancho en `14` caracteres de la mono y pinta el borde `PELIGRO` si el valor es inválido.

- [ ] **Step 5: Test de interacción de `tab_bar`**

```rust
// crates/app/tests/kit.rs
use egui_kittest::kittest::Queryable; // egui_kittest re-exporta el crate kittest
use rod_app::editor_app::Panel;

#[test]
fn tab_bar_devuelve_la_pestana_pulsada() {
    let pulsada = std::cell::Cell::new(None);
    let mut harness = egui_kittest::Harness::builder().build_ui(|ui| {
        if let Some(p) = rod_app::ui::kit::tab_bar(ui, &[(Panel::Hablantes, "Hablantes", Some(3))], Panel::Subtitulos) {
            pulsada.set(Some(p));
        }
    });
    harness.get_by_label("Hablantes").click();
    harness.run();
    assert_eq!(pulsada.get(), Some(Panel::Hablantes));
}
```

- [ ] **Step 6: Ejecutar todos los tests del kit**

Run: `cargo test -p rod-app iconos kit`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/app/Cargo.toml Cargo.lock crates/app/src/ui/kit.rs crates/app/src/ui/iconos.rs crates/app/src/ui/mod.rs crates/app/tests/kit.rs
git commit -m "feat: kit de componentes e iconos sin tofu"
```

---

### Task 5: Textos ES/EN

**Files:**
- Create: `crates/app/src/textos.rs`
- Modify: `crates/app/src/lib.rs`, `crates/app/src/comandos.rs` (quitar `etiqueta`)

**Interfaces:**
- Produces: `pub enum Idioma { Es, En }` con `Default` (inglés) y `fn textos(self) -> &'static Textos`; `pub struct Textos` con `pub const ES: Textos` y `pub const EN: Textos`; métodos `fn comando(&self, Comando) -> &'static str`, `fn lineas(&self, usize) -> String`, `fn hablantes(&self, usize) -> String`, `fn fecha(&self, FechaRelativa) -> String`.

- [ ] **Step 1: Test de cobertura y de idioma**

```rust
#[test]
fn todo_comando_tiene_etiqueta_en_los_dos_idiomas() {
    for comando in Comando::todas() {
        assert!(!ES.comando(*comando).is_empty(), "{comando:?} en ES");
        assert!(!EN.comando(*comando).is_empty(), "{comando:?} en EN");
    }
}

#[test]
fn el_idioma_por_defecto_es_ingles() {
    assert_eq!(Idioma::default(), Idioma::En);
    assert_eq!(EN.inicio_titulo, "Edit subtitles with precision");
    assert_eq!(ES.inicio_titulo, "Edita subtítulos con precisión");
}

#[test]
fn plurales_en_los_dos_idiomas() {
    assert_eq!(EN.lineas(1), "1 line");
    assert_eq!(EN.lineas(543), "543 lines");
    assert_eq!(ES.lineas(1), "1 línea");
    assert_eq!(ES.lineas(543), "543 líneas");
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app textos`
Expected: FAIL, módulo inexistente.

- [ ] **Step 3: Escribir las dos tablas**

Empieza con las frases de Inicio, la cabecera y los comandos; cada tarea siguiente añade las suyas a **las dos** tablas en el mismo commit. `comando` es un `match` exhaustivo por tabla.

- [ ] **Step 4: Ejecutar los tests**

Run: `cargo test -p rod-app textos`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/textos.rs crates/app/src/lib.rs crates/app/src/comandos.rs
git commit -m "feat: textos en ingles y espanol con cobertura por tipo"
```

---

### Task 6: Atajos configurables y preferencias

**Files:**
- Modify: `crates/app/src/comandos.rs` (reescrito parcialmente), `crates/app/src/lib.rs`, `crates/app/src/main.rs`
- Create: `crates/app/src/preferencias.rs`

**Interfaces:**
- Produces: `pub struct Atajo { pub tecla: egui::Key, pub ctrl: bool, pub shift: bool, pub alt: bool }` con `Display`; `pub struct MapaAtajos`; `MapaAtajos::por_defecto()`, `.comando_de(Key, Modifiers, foco_en_texto: bool) -> Option<Comando>`, `.atajo(Comando) -> Option<Atajo>`, `.asignar(Comando, Atajo) -> Result<(), Conflicto>`, `.intercambiar(Comando, Comando)`, `.restablecer(Comando)`, `.restablecer_todo()`; `Comando::todas() -> &'static [Comando]`; `pub struct Preferencias { pub ancho_panel: f32, pub alto_timeline: f32, pub panel_visible: bool, pub idioma: Idioma, pub atajos: MapaAtajos }` con `Default`, `pub fn ruta() -> PathBuf`, `pub fn cargar() -> Preferencias`, `pub fn guardar(&self) -> std::io::Result<()>`, y para los tests `cargar_en(ruta: &Path) -> Preferencias` / `guardar_en(&self, ruta: &Path) -> std::io::Result<()>`.

- [ ] **Step 1: Test de paridad con el comportamiento actual**

```rust
#[test]
fn el_mapa_por_defecto_reproduce_los_atajos_de_hoy() {
    let mapa = MapaAtajos::por_defecto();
    for (tecla, ctrl, shift, alt, esperado) in [
        (Key::S, true, false, false, Comando::Guardar),
        (Key::S, true, true, false, Comando::GuardarComo),
        (Key::Z, true, false, false, Comando::Deshacer),
        (Key::Space, false, false, false, Comando::ReproducirPausa),
        (Key::ArrowLeft, false, false, true, Comando::CaptionAnterior),
        (Key::ArrowLeft, false, true, false, Comando::RetrocederFino),
        (Key::C, false, false, false, Comando::Dividir),
        (Key::Num7, false, false, false, Comando::Hablante(7)),
    ] {
        let mods = Modifiers { ctrl, command: ctrl, shift, alt, ..Default::default() };
        assert_eq!(mapa.comando_de(tecla, mods, false), Some(esperado), "{tecla:?}");
    }
}

#[test]
fn con_foco_en_un_campo_las_teclas_desnudas_no_disparan() {
    let mapa = MapaAtajos::por_defecto();
    let mods = Modifiers::default();
    assert_eq!(mapa.comando_de(Key::Space, mods, true), None);
    assert_eq!(mapa.comando_de(Key::C, mods, true), None);
    assert_eq!(mapa.comando_de(Key::Num1, mods, true), None);
    assert_eq!(mapa.comando_de(Key::S, Modifiers { ctrl: true, command: true, ..Default::default() }, true), Some(Comando::Guardar));
}

#[test]
fn no_hay_atajos_repetidos_y_asignar_uno_tomado_falla() {
    let mut mapa = MapaAtajos::por_defecto();
    assert_eq!(mapa.comando_de(Key::S, Modifiers { ctrl: true, command: true, ..Default::default() }, false), Some(Comando::Guardar));
    assert!(mapa.asignar(Comando::Deshacer, Atajo { tecla: Key::S, ctrl: true, shift: false, alt: false }).is_err());
    mapa.intercambiar(Comando::Guardar, Comando::Deshacer);
    assert_eq!(mapa.atajo(Comando::Deshacer).unwrap().to_string(), "Ctrl+S");
}
```

- [ ] **Step 2: Tests de preferencias**

```rust
#[test]
fn preferencias_rotas_dan_los_valores_de_fabrica() {
    let dir = std::env::temp_dir().join(format!("rod-prefs-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let ruta = dir.join("preferencias.json");
    std::fs::write(&ruta, "{ no es json").unwrap();
    let prefs = Preferencias::cargar_en(&ruta);
    assert_eq!(prefs.idioma, Idioma::En);
    assert_eq!(prefs.atajo(Comando::Guardar).unwrap().to_string(), "Ctrl+S");
}

#[test]
fn preferencias_hacen_round_trip() {
    let mut prefs = Preferencias::default();
    prefs.idioma = Idioma::Es;
    prefs.ancho_panel = 348.0;
    prefs.atajos = MapaAtajos::por_defecto();
    // guardar_en + cargar_en → mismos valores
}
```

- [ ] **Step 3: Ejecutar y ver los fallos**

Run: `cargo test -p rod-app comandos preferencias`
Expected: FAIL por tipos ausentes.

- [ ] **Step 4: Implementar el mapa, `todas()` y `preferencias`**

`comando_de` compara `Atajo` y aplica la regla de foco. `Preferencias::guardar` escribe a `.tmp` y renombra, como `recientes.rs`. Añadir `cargar_en`/`guardar_en` públicos para los tests. `main.rs` sustituye `desde_input(...)` por `self.preferencias.atajos.comando_de(...)` y lee el foco con `ctx.memory(|m| m.focused().is_some())`.

- [ ] **Step 5: Ejecutar todos los tests**

Run: `cargo test -p rod-app comandos preferencias`
Expected: PASS. Los tests viejos de `comandos.rs` que llaman `desde_input` se reescriben sobre el mapa, conservando el caso `ctrl_s_y_ctrl_f_funcionan_dentro_de_un_campo`.

- [ ] **Step 6: Commit**

```bash
git add crates/app/src/comandos.rs crates/app/src/preferencias.rs crates/app/src/lib.rs crates/app/src/main.rs
git commit -m "feat: atajos configurables y preferencias persistidas"
```

---

### Task 7: Repintado condicional

**Files:**
- Create: `crates/app/src/ui/repintado.rs`
- Modify: `crates/app/src/main.rs`, `crates/app/src/ui/mod.rs`

**Interfaces:**
- Produces: `pub fn politica(reproduciendo: bool, sucio: bool, buscando: bool) -> Option<std::time::Duration>`.

- [ ] **Step 1: Tests de la política**

```rust
#[test]
fn en_reposo_limpio_no_pide_frames() {
    assert_eq!(politica(false, false, false), None);
}

#[test]
fn con_cambios_sin_guardar_late_a_un_segundo() {
    assert_eq!(politica(false, true, false), Some(Duration::from_secs(1)));
}

#[test]
fn buscando_late_rapido_y_reproduciendo_cada_frame() {
    assert_eq!(politica(false, false, true), Some(Duration::from_millis(100)));
    assert_eq!(politica(true, false, false), Some(Duration::ZERO));
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app repintado`
Expected: FAIL.

- [ ] **Step 3: Implementar y usarla en `main.rs`**

Sustituir el `ctx.request_repaint()` incondicional por `match politica(...)`: `Some(Duration::ZERO) => ctx.request_repaint()`, `Some(d) => ctx.request_repaint_after(d)`, `None => {}`. `buscando` = `editor.player.seeking()`; `sucio` = `editor_app::sucio(editor)`.

- [ ] **Step 4: Verificar a mano que no se rompe nada**

Run: `cargo test -p rod-app repintado && cargo clippy -p rod-app -- -D warnings`
Expected: PASS. En la app real (cuando se pueda abrir), la reproducción sigue fluida y el autosave sigue escribiendo.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/ui/repintado.rs crates/app/src/ui/mod.rs crates/app/src/main.rs
git commit -m "perf: repintar solo cuando hay algo que ver"
```

---

### Task 8: Chrome y menús

**Files:**
- Create: `crates/app/src/ui/chrome.rs`
- Modify: `crates/app/src/main.rs`, `crates/app/src/menu.rs`, `crates/app/src/comandos.rs` (sin cambios de lógica, solo presentación desde `Textos`)

**Interfaces:**
- Produces: `pub struct EstadoCabecera<'a> { pub en_inicio: bool, pub proyecto: Option<&'a str>, pub sucio: bool }`; `pub fn mostrar(ctx: &egui::Context, textos: &Textos, atajos: &MapaAtajos, estado: EstadoCabecera) -> Option<Comando>`.
- Consumes: `kit::icon_button`, `comandos::{seccion, disponible_en_inicio}`, `textos::Textos::comando`.

- [ ] **Step 1: Test de que el atajo no viaja dentro de la etiqueta**

```rust
// crates/app/tests/chrome.rs
#[test]
fn el_menu_separa_etiqueta_y_atajo() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui(|ui| {
            rod_app::ui::chrome::mostrar_menu_para_test(ui, &rod_app::textos::EN, &MapaAtajos::por_defecto());
        });
    harness.get_by_label("Archivo").click();
    harness.run();
    harness.get_by_label("Guardar proyecto");
    assert!(harness.query_by_label("Guardar proyecto   Ctrl+S").is_none());
    harness.get_by_label("Ctrl+S");
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app --test chrome`
Expected: FAIL.

- [ ] **Step 3: Implementar la cabecera**

40 px con `Margin::symmetric(ESP_12, 0)`: monograma de 24 con tres barras en `#F2B84B`, `#B39BFF`, `#3FC4B8`; «OpenDialogue» en `fuente(Estilo::CuerpoFuerte)`; separador; menús; franja arrastrable; nombre + estado centrados con rect calculado; tres botones de 46×40 con iconos, el de cerrar con hover `PELIGRO`. Los menús se pintan con `egui::menu::bar` + `MenuButton` de altura 24 y una rejilla de dos columnas por ítem (etiqueta izquierda, atajo en mono derecha).

- [ ] **Step 4: Ejecutar el test y las capturas**

Run: `cargo test -p rod-app --test chrome && cargo run -p rod-app --example capturas -- fase2-cabecera`
Expected: PASS y captura con la cabecera nueva.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/ui/chrome.rs crates/app/src/main.rs crates/app/src/menu.rs crates/app/tests/chrome.rs docs/checklists/rediseno
git commit -m "feat: cabecera con monograma y menus con atajos alineados"
```

---

### Task 9: Inicio

**Files:**
- Rewrite: `crates/app/src/ui/inicio.rs`
- Modify: `crates/app/src/inicio.rs` (orden y destacado), `crates/app/src/main.rs`

**Interfaces:**
- Consumes: `Reciente` ampliado (Task 10), `kit::*`, `textos::Textos`, `ui::ajustes` (Task 11) para el riel.
- Produces: `pub fn mostrar(ui: &mut Ui, estado: &mut EstadoInicio, textos: &Textos, prefs: &mut Preferencias) -> (Option<MensajeInicio>, bool)` donde el `bool` es «abrir Ajustes».

- [ ] **Step 1: Test de la rejilla, el vacío y el drop**

```rust
// crates/app/tests/inicio.rs
#[test]
fn tres_recientes_dan_tres_tarjetas() {
    let mut estado = estado_con_tres();
    let mut harness = Harness::builder().with_size(vec2(1440.0, 900.0))
        .build_ui(|ui| { rod_app::ui::inicio::mostrar(ui, &mut estado, &EN, &mut Preferencias::default()); });
    harness.get_by_label("RobloxTerror.cdp");
    harness.get_by_label("543 lines · 7 speakers");
}

#[test]
fn sin_recientes_muestra_el_vacio() {
    let mut estado = EstadoInicio::nuevo(vec![], None);
    /* … */
    harness.get_by_label(&EN.texto_vacio_recientes);
}

#[test]
fn el_riel_marca_ajustes_cuando_esta_abierto() { /* … */ }
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app --test inicio`
Expected: FAIL.

- [ ] **Step 3: Implementar riel, columna y tarjetas**

Riel de 220 con Inicio, Recientes y Ajustes (este devuelve `true`); columna de 960 centrada con 72 de aire; título `Estilo::Titulo`; botones primario y secundario con `Ctrl+O` en mono; zona de arrastre de 104 con `ui.ctx().input(|i| i.raw.hovered_files)` y reacción de color; etiqueta «Recientes» con ComboBox de orden; rejilla calculada a mano con mínimo 260 y separación 20; tarjeta de Task 10; fantasma punteada; estado vacío; banda de error con `PELIGRO` al 12 %.

- [ ] **Step 4: Generar capturas y compararlas con el mockup**

Run: `cargo run -p rod-app --example capturas -- fase2-inicio`
Expected: `fase2-inicio-*.png` con riel, columna de 960, tarjetas de 3 columnas, tira de hablantes y chip de duración. Comparar contra el mockup abierto al lado: mismos márgenes (72, 24, 20) y mismos tamaños de texto.

- [ ] **Step 5: Tests y commit**

Run: `cargo test -p rod-app --test inicio`
Expected: PASS.

```bash
git add crates/app/src/ui/inicio.rs crates/app/src/inicio.rs crates/app/src/main.rs crates/app/tests/inicio.rs docs/checklists/rediseno
git commit -m "feat: pantalla de Inicio con riel, drop real y tarjetas"
```

---

### Task 10: Reciente ampliado y fechas traducibles

**Files:**
- Modify: `crates/app/src/recientes.rs`, `crates/app/src/inicio.rs`, `crates/app/src/textos.rs`

**Interfaces:**
- Produces: `pub struct FechaRelativa { pub dias: u64 }`; `pub fn fecha_relativa(mtime_ms: f64, ahora_ms: f64) -> FechaRelativa`; `Reciente { ruta, lineas, hablantes, fecha: FechaRelativa, duracion: f64, tira: Vec<(String, f32)> }`; `Textos::fecha(&FechaRelativa) -> String`.

- [ ] **Step 1: Tests de metadata**

```rust
#[test]
fn la_duracion_es_el_fin_del_ultimo_caption() {
    let r = super::metadata(&ruta_fixture(), 0.0).unwrap();
    assert!((r.duracion - 4.0).abs() < 1e-6);
}

#[test]
fn la_tira_suma_los_segundos_por_hablante_y_omite_a_quien_no_habla() {
    let r = super::metadata(&ruta_fixture(), 0.0).unwrap();
    assert_eq!(r.tira.len(), 2);
    assert!((r.tira[0].1 - 1.0).abs() < 1e-6);
    assert_ne!(r.tira[0].0, r.tira[1].0, "dos hablantes con el mismo color");
}

#[test]
fn la_fecha_no_lleva_texto() {
    assert_eq!(super::fecha_relativa(1_000.0, 1_000.0), FechaRelativa { dias: 0 });
    assert_eq!(EN.fecha(&FechaRelativa { dias: 1 }), "yesterday");
    assert_eq!(ES.fecha(&FechaRelativa { dias: 3 }), "hace 3 días");
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app recientes`
Expected: FAIL.

- [ ] **Step 3: Implementar**

`metadata` ya lee el `.cdp`: de ahí salen `duracion` (máximo `fin`) y `tira` (suma de `fin - inicio` por `hablante_id`, con `color_identidad` y el índice del hablante). `fecha_relativa` deja de devolver `String`. `inicio::resumen_reciente` y `texto_vacio_recientes` se borran; su texto pasa a `Textos`.

- [ ] **Step 4: Ejecutar y ajustar llamadas**

`Reciente.fecha` pasa de `String` a `FechaRelativa`, así que **todas** las
construcciones literales del struct cambian: los tests de `inicio.rs`, los de
`recientes.rs` y los tres recientes del ejemplo de capturas. El texto ya no se
guarda en el struct: se calcula al dibujar con `Textos::fecha`.

Run: `cargo test -p rod-app recientes inicio && cargo run -p rod-app --example capturas -- fase2-inicio`
Expected: PASS tras actualizar `ui/inicio.rs`, `ui/paneles` y el ejemplo a las
nuevas firmas.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/recientes.rs crates/app/src/inicio.rs crates/app/src/textos.rs crates/app/src/ui/inicio.rs
git commit -m "feat: recientes con duracion y tira de hablantes, fechas sin idioma fijo"
```

---

### Task 11: Capa de Ajustes (idioma y atajos)

**Files:**
- Create: `crates/app/src/ui/ajustes.rs`
- Modify: `crates/app/src/ui/mod.rs`, `crates/app/src/main.rs`, `crates/app/src/comandos.rs` (conflictos), `crates/app/src/textos.rs`

**Interfaces:**
- Produces: `pub struct EstadoAjustes { pub abierto: bool, pub pestana: Pestana, pub capturando: Option<Comando>, pub busqueda: String }` con `Default`; `pub fn mostrar(ctx: &egui::Context, prefs: &mut Preferencias, estado: &mut EstadoAjustes) -> bool` (devuelve `true` si algo cambió y hay que guardar).

- [ ] **Step 1: Test de cambio de idioma y de rebinding**

```rust
// crates/app/tests/ajustes.rs
#[test]
fn cambiar_a_espanol_se_ve_al_instante() {
    let mut prefs = Preferencias::default();
    let mut estado = EstadoAjustes { abierto: true, ..Default::default() };
    let mut harness = Harness::builder().with_size(vec2(1440.0, 900.0))
        .build_ui(|ctx| { rod_app::ui::ajustes::mostrar(ctx, &mut prefs, &mut estado); });
    harness.get_by_label("Español").click();
    harness.run();
    assert_eq!(prefs.idioma, Idioma::Es);
    harness.get_by_label("Atajos"); // ya en español
}

#[test]
fn reasignar_una_tecla_tomada_pide_intercambio() {
    // pulsar «cambiar» en Deshacer, pulsar Ctrl+S, comprobar que aparece
    // «Intercambiar» y que tras pulsarlo Deshacer usa Ctrl+S y Guardar Ctrl+Z
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app --test ajustes`
Expected: FAIL.

- [ ] **Step 3: Implementar la capa**

Modal 720×560 sobre velo al 45 %, `tab_bar` con Atajos e Idioma, lista de `Comando::todas()` agrupada por `Seccion` con buscador, captura de tecla desde `ctx.input(|i| i.events)`, conflicto con botones «Intercambiar»/«Cancelar», restablecer por fila y global. Idioma con dos radios. `Esc` y clic fuera cierran; `mem::discard` del foco al cerrar.

- [ ] **Step 4: Enganchar en `main.rs`**

El riel y el menú Edición ponen `estado.abierto = true`. Si `mostrar` devuelve `true`, `preferencias.guardar()`.

- [ ] **Step 5: Tests y captura**

Run: `cargo test -p rod-app --test ajustes && cargo run -p rod-app --example capturas -- fase2-ajustes`
Expected: PASS y captura de la capa en las dos pestañas.

- [ ] **Step 6: Commit**

```bash
git add crates/app/src/ui/ajustes.rs crates/app/src/ui/mod.rs crates/app/src/main.rs crates/app/src/comandos.rs crates/app/src/textos.rs crates/app/tests/ajustes.rs docs/checklists/rediseno
git commit -m "feat: ajustes con idioma y atajos, persistidos al instante"
```

---

### Task 12: Distribución del editor

**Files:**
- Rewrite: `crates/app/src/ui/editor.rs`
- Modify: `crates/app/src/main.rs`, `crates/app/src/editor_app.rs` (aplicar preferencias)

**Interfaces:**
- Produces: `pub struct MedidasEditor { pub ancho_panel: f32, pub alto_timeline: f32, pub panel_visible: bool }`; `pub fn show(ctx, state: &mut EditorApp, formato, gpu, textos: &Textos, atajos: &MapaAtajos, medidas: &mut MedidasEditor) -> Vec<MensajeEditor>`.

- [ ] **Step 1: Test de que el editor se renderiza con el fixture**

```rust
// crates/app/tests/editor.rs
fn editor_de_prueba() -> EditorApp {
    comun::editor_543() // ver «Fixture compartida»
}

#[test]
fn el_editor_pinta_monitor_panel_y_timeline() {
    let mut app = editor_de_prueba();
    let mut medidas = MedidasEditor::default();
    let mut harness = Harness::builder().with_size(vec2(1440.0, 900.0))
        .build_ui(|ctx| { rod_app::ui::editor::show(ctx, &mut app, wgpu::TextureFormat::Bgra8Unorm, std::sync::Arc::new(std::sync::Mutex::new(None)), &EN, &MapaAtajos::por_defecto(), &mut medidas); });
    harness.run();
    harness.get_by_label("Subtítulos");
    harness.get_by_label("Hablantes");
    harness.get_by_label("Estilos");
}
```

Si mpv no abre sin ventana en el arnés, el test se marca `#[ignore]` con el motivo y la captura del editor se hace desde la app real cuando el usuario lo permita; se anota en `BITACORA.md`. Las tareas 13–18 se prueban en su lugar con tests unitarios de sus funciones puras y con `Harness` sobre el panel aislado.

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app --test editor`
Expected: FAIL.

- [ ] **Step 3: Implementar la distribución**

`SidePanel::right` 320 (280–420) con `panel_header` y `tab_bar`; `TopBottomPanel::bottom` 260 (160–60 %) con la cabecera del timeline; `CentralPanel` con monitor (relación de aspecto de `ui/video.rs`) y transporte de 56; barra de estado de 24. Doble clic en cada divisor colapsa. `medidas` se rellena desde los `rect` de los paneles. `main.rs` aplica `medidas` a `Preferencias` al arrancar y la guarda con debounce de 1 s.

- [ ] **Step 4: Capturas del editor**

Run: `cargo run -p rod-app --example capturas -- fase3-editor`
Expected: captura headless a 1440×900 con panel y timeline nuevos (monitor vacío, es esperado); la captura del editor con video se hace con `capturar.ps1`, como en la tarea 1.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/ui/editor.rs crates/app/src/main.rs crates/app/src/editor_app.rs crates/app/tests/editor.rs docs/checklists/rediseno
git commit -m "feat: distribucion del editor con paneles redimensionables y persistidos"
```

---

### Task 13: Lista de subtítulos y edición inline

**Files:**
- Rewrite: `crates/app/src/ui/paneles/subtitulos.rs`
- Modify: `crates/app/src/editor_app.rs` (mensajes de edición inline), `crates/app/src/textos.rs`

**Interfaces:**
- Consumes: `kit::{punto, chip_hablante}`, `tema::ALTO_FILA_SUB`, `Textos`.
- Produces: `pub fn mostrar(ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor>`; `MensajeEditor::EditarInline(String, Option<String>)` y `MensajeEditor::CancelarInline`.

- [ ] **Step 1: Test de doble clic, Enter y Esc**

```rust
#[test]
fn doble_clic_abre_editor_y_escape_no_cambia_el_texto() {
    let mut app = editor_de_prueba();
    let original = app.editor.proyecto().captions[0].texto.clone();
    /* harness con el panel de subtítulos, doble clic en la fila 0, escribir "x", Escape */
    harness.run();
    assert_eq!(app.editor.proyecto().captions[0].texto, original);
    assert!(!app.texto_inline_activo());
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app --test subtitulos`
Expected: FAIL.

- [ ] **Step 3: Implementar la lista**

`ScrollArea::vertical().auto_shrink([false; 2])` + `show_rows(ui, ALTO_FILA_SUB, n, …)`: barra de 3 px del color de identidad, dos líneas (texto 13 con elipsis y tooltip; timecode mono 13 + duración + chip), selección con `ACENTO` al 18 %, `Ctrl`/`Shift` clic, flechas con desplazamiento, doble clic a edición inline con `TextEdit` que confirma con Enter y cancela con Esc.

- [ ] **Step 4: Tests y commit**

Run: `cargo test -p rod-app --test subtitulos`
Expected: PASS.

```bash
git add crates/app/src/ui/paneles/subtitulos.rs crates/app/src/editor_app.rs crates/app/src/textos.rs crates/app/tests/subtitulos.rs
git commit -m "feat: lista de subtitulos virtualizada con edicion inline"
```

---

### Task 14: Inspector y toasts

**Files:**
- Modify: `crates/app/src/ui/paneles/subtitulos.rs`, `crates/app/src/editor_app.rs`, `crates/app/src/textos.rs`
- Create: `crates/app/src/ui/toast.rs`

**Interfaces:**
- Produces: `pub fn inspector(ui, state: &mut EditorApp, textos: &Textos) -> Vec<MensajeEditor>`; `pub struct Toasts` con `pub fn push(&mut self, texto: String, deshacer: bool)`, `pub fn mostrar(&mut self, ctx: &Context, textos: &Textos) -> bool` (devuelve si se pulsó «Deshacer»).

- [ ] **Step 1: Test de aplicar texto con Enter desde el inspector**

```rust
#[test]
fn escribir_en_el_inspector_y_enter_aplica() {
    /* harness con inspector, seleccionar la fila 0, escribir "Hola", Enter */
    harness.run();
    assert_eq!(app.editor.proyecto().captions[0].texto, "Hola");
}

#[test]
fn borrar_muestra_toast_y_deshacer_restaura() {
    /* seleccionar, pulsar papelera, comprobar que hay un toast con "Deshacer",
       pulsarlo y comprobar que el caption vuelve */
    assert_eq!(app.editor.proyecto().captions.len(), 543);
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app --test inspector`
Expected: FAIL.

- [ ] **Step 3: Implementar el inspector y borrar el bloque viejo**

Textarea auto-creciente (3–8 líneas) con `Borrador`/`AplicarTexto`; fila de hablante con desplegable de 12 muestras + «Sin hablante»; entrada/salida con `campo_tiempo` y botones ±1 cuadro; papelera; todo en vivo. Eliminar el bloque TEXTO/ENTRADA/SALIDA/Aplicar/Borrar y su test de `ui/editor.rs`.

- [ ] **Step 4: Implementar `Toasts`**

Abajo al centro, 6 s, icono por tono, botón «Deshacer» que emite `MensajeEditor::Deshacer`.

- [ ] **Step 5: Tests y commit**

Run: `cargo test -p rod-app --test inspector`
Expected: PASS.

```bash
git add crates/app/src/ui/paneles/subtitulos.rs crates/app/src/ui/toast.rs crates/app/src/editor_app.rs crates/app/src/textos.rs crates/app/tests/inspector.rs
git commit -m "feat: inspector de subtitulo y toasts con deshacer"
```

---

### Task 15: Identidad de color de hablante

**Files:**
- Modify: `crates/core/src/color_hablante.rs`, `crates/core/src/lib.rs`, `crates/app/src/ui/paneles/hablantes.rs`, `crates/app/src/textos.rs`

**Interfaces:**
- Produces: `rod_core::paleta_identidad() -> &'static [&'static str]` (12), `rod_core::color_identidad(hablante: &Hablante, indice: usize) -> &'static str`, `rod_core::es_color_de_paleta(hex: &str) -> bool`.
- `color_hablante` (preset) no cambia.

- [ ] **Step 1: Tests en `rod-core`**

```rust
#[test]
fn la_paleta_tiene_doce_colores_unicos_y_ninguno_es_el_acento() {
    let paleta = paleta_identidad();
    assert_eq!(paleta.len(), 12);
    let mut vistos = std::collections::HashSet::new();
    for color in paleta {
        assert!(vistos.insert(*color), "{color} repetido");
        assert_ne!(*color, "#3B8EFF");
    }
}

#[test]
fn un_color_de_paleta_guardado_manda_sobre_el_indice() {
    let h = hablante(Some("#F27BB5"));
    assert_eq!(color_identidad(&h, 0), "#F27BB5");
    assert_eq!(color_identidad(&h, 5), "#F27BB5");
}

#[test]
fn un_color_invalido_o_blanco_cae_al_indice_sin_repetir_en_los_doce_primeros() {
    for i in 0..12 {
        let color = color_identidad(&hablante(None), i);
        assert!(es_color_de_paleta(color));
        assert_ne!(color, color_identidad(&hablante(None), (i + 1) % 12));
    }
    assert_eq!(color_identidad(&hablante(None), 12), color_identidad(&hablante(None), 0));
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-core color_hablante`
Expected: FAIL.

- [ ] **Step 3: Implementar en `rod-core`**

Doce hex de la spec §2.4 en el orden dado. Sin dependencias nuevas, sin egui.

- [ ] **Step 4: Reescribir el panel Hablantes**

Tarjeta por hablante: nombre inline, tecla en mono, 12 muestras (`icon_button` de 24 con borde en el color, marca en el elegido), chip de identidad, chip del color del preset con enlace a Estilos (`MensajeEditor::Panel(Panel::Estilos)` + `SeleccionarPreset`), contador de líneas, papelera con confirmación, «Agregar hablante».

- [ ] **Step 5: Test de que el color no se puede sacar de la paleta**

```rust
#[test]
fn el_selector_solo_ofrece_los_doce_colores() {
    /* harness con el panel, contar botones de muestra, pulsar el 4.º
       y comprobar que el hablante guardó exactamente ese hex */
    assert_eq!(app.editor.proyecto().hablantes[0].color, rod_core::paleta_identidad()[3]);
}
```

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/color_hablante.rs crates/core/src/lib.rs crates/app/src/ui/paneles/hablantes.rs crates/app/src/textos.rs crates/app/tests/hablantes.rs
git commit -m "feat: identidad de color por hablante separada del color del preset"
```

---

### Task 16: Waveform

**Files:**
- Create: `crates/player/src/waveform.rs`
- Modify: `crates/player/Cargo.toml`, `Cargo.toml`, `crates/player/src/lib.rs`, `crates/app/src/editor_app.rs`

**Interfaces:**
- Produces: `pub struct Picos { pub por_segundo: u32, pub max: Vec<f32>, pub rms: Vec<f32> }`; `pub fn analizar(ruta: &Path) -> Result<Picos, ErrorWaveform>`; `pub fn analizar_en_hilo(ruta: PathBuf) -> std::sync::mpsc::Receiver<Result<Picos, ErrorWaveform>>`.

- [ ] **Step 1: Test con el clip del repo**

```rust
#[test]
fn el_clip_de_prueba_da_picos_proporcionales_a_su_duracion() {
    let picos = analizar(Path::new("../../testdata/clip-15s-2audio.mp4")).expect("decodifica");
    assert_eq!(picos.por_segundo, 100);
    assert!(picos.max.len() > 10 * picos.por_segundo as usize);
    assert!(picos.max.iter().any(|v| *v > 0.05), "todo silencio");
}

#[test]
fn un_archivo_que_no_es_audio_da_error_sin_panico() {
    let dir = std::env::temp_dir().join("rod-waveform");
    std::fs::create_dir_all(&dir).unwrap();
    let ruta = dir.join("no-es-audio.mp4");
    std::fs::write(&ruta, b"basura").unwrap();
    assert!(analizar(&ruta).is_err());
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-player waveform`
Expected: FAIL.

- [ ] **Step 3: Añadir symphonia y decodificar**

`symphonia = { version = "0.5", default-features = false, features = ["isomp4", "aac", "mp3", "wav", "flac", "std"] }`. Decodificar la primera pista, mezclar a mono, acumular máximo y RMS por ventana de 10 ms.

- [ ] **Step 4: Pedirlo al abrir video**

`EditorApp` guarda `picos: Option<Picos>` y el `Receiver`; en `Tick` hace `try_recv`. Sin picos, el timeline pinta la pista vacía.

- [ ] **Step 5: Tests y commit**

Run: `cargo test -p rod-player waveform && cargo test -p rod-app`
Expected: PASS.

```bash
git add Cargo.toml crates/player/Cargo.toml crates/player/src/waveform.rs crates/player/src/lib.rs crates/app/src/editor_app.rs
git commit -m "feat: waveform decodificada con symphonia en un hilo aparte"
```

---

### Task 17: Timeline

**Files:**
- Rewrite: `crates/app/src/ui/timeline.rs`
- Modify: `crates/app/src/timeline.rs` (funciones puras de regla, ajuste y zoom), `crates/app/src/textos.rs`

**Interfaces:**
- Produces (en `timeline.rs`, puras y testeables): `pub fn paso_de_regla(px_por_segundo: f64) -> f64`; `pub fn iman(candidato: f64, puntos: &[f64], tolerancia_s: f64) -> Option<f64>`; `pub fn tiempo_bajo_cursor(vista: Vista, x: f32) -> f64`; `pub fn escala_anclada(vista: Vista, factor: f64, x: f32, duracion: f64, ancho: f32) -> Vista`.
- `pub fn mostrar(ui, state: &mut EditorApp, textos: &Textos, atajos: &MapaAtajos) -> Vec<MensajeEditor>`.

- [ ] **Step 1: Tests de las funciones puras**

```rust
#[test]
fn la_regla_cambia_de_paso_segun_el_zoom() {
    assert_eq!(paso_de_regla(10.0), 10.0);
    assert_eq!(paso_de_regla(100.0), 1.0);
    assert_eq!(paso_de_regla(1000.0), 0.5);
}

#[test]
fn el_iman_pega_al_borde_mas_cercano_dentro_de_la_tolerancia() {
    assert_eq!(iman(4.98, &[0.0, 5.0, 9.0], 0.06), Some(5.0));
    assert_eq!(iman(4.80, &[0.0, 5.0, 9.0], 0.06), None);
}

#[test]
fn el_zoom_conserva_el_tiempo_bajo_el_cursor() {
    let vista = Vista { inicio: 0.0, escala: 0.1 };
    let antes = tiempo_bajo_cursor(vista, 400.0);
    let despues = escala_anclada(vista, 2.0, 400.0, 20.0, 800.0);
    assert!((tiempo_bajo_cursor(despues, 400.0) - antes).abs() < 1e-6);
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app timeline`
Expected: FAIL.

- [ ] **Step 3: Implementar el painter**

Cabeceras de 120, pista de audio de 44 con la waveform (`#3A4A5A`), una pista por hablante con mute/solo/candado, fila «Sin hablante» con relleno `CONTROL` y borde discontinuo; clip al 22 % con borde al 100 %, handles de 6 px, selección con borde `ACENTO` de 2 px, solape del mismo hablante en `PELIGRO`, playhead con cabeza triangular y chip de timecode al arrastrar; regla de 28; marquesina; `Ctrl+rueda` con `escala_anclada`, rueda con desplazamiento, `Shift+rueda` vertical; solo se pintan los clips visibles.

- [ ] **Step 4: Test de que un clip se mueve con el ratón**

```rust
// crates/app/tests/timeline.rs (con Harness sobre el timeline del fixture)
#[test]
fn arrastrar_un_clip_cambia_sus_tiempos_y_deja_un_paso_de_deshacer() {
    /* arrastrar el clip 0 hacia la derecha 40 px, soltar */
    assert!(app.editor.proyecto().captions[0].inicio > 1.0);
    app.editor.deshacer().unwrap();
    assert_eq!(app.editor.proyecto().captions[0].inicio, 1.0);
}
```

- [ ] **Step 5: Captura y commit**

Run: `cargo run -p rod-app --example capturas -- fase4-timeline && cargo test -p rod-app timeline`
Expected: PASS y captura del timeline con waveform, pistas y clips de colores.

```bash
git add crates/app/src/ui/timeline.rs crates/app/src/timeline.rs crates/app/src/textos.rs crates/app/tests/timeline.rs docs/checklists/rediseno
git commit -m "feat: timeline pintado a mano con waveform, zoom anclado y ajuste"
```

---

### Task 18: Transporte y barra de estado

**Files:**
- Rewrite: `crates/app/src/ui/transporte.rs`
- Modify: `crates/app/src/ui/editor.rs`, `crates/app/src/textos.rs`

**Interfaces:**
- Produces: `pub fn mostrar(ui, state: &mut EditorApp, textos: &Textos, atajos: &MapaAtajos) -> Vec<MensajeEditor>`; `pub fn barra_estado(ui, state: &EditorApp, textos: &Textos)`.

- [ ] **Step 1: Test del timecode y del botón de play**

```rust
#[test]
fn el_timecode_sale_de_format_time_y_el_play_mide_36() {
    /* harness sobre el transporte con position = 19.48 s */
    harness.get_by_label("00:00:19:12");
    let r = harness.get_by_label("Reproducir");
    assert!((r.rect().height() - 36.0).abs() < 0.5);
}
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app --test transporte`
Expected: FAIL.

- [ ] **Step 3: Implementar**

Iconos de inicio, cuadro anterior, play de 36, cuadro siguiente, fin; timecode grande en mono 20 clicable; total en `Pequeno`; `ComboBox` de velocidad; volumen; bucle. Barra de estado `N líneas · M hablantes · S solapes` con el conteo de `exportar_srt_por_hablante` y el estado de guardado.

- [ ] **Step 4: Tests y commit**

Run: `cargo test -p rod-app --test transporte`
Expected: PASS.

```bash
git add crates/app/src/ui/transporte.rs crates/app/src/ui/editor.rs crates/app/src/textos.rs crates/app/tests/transporte.rs
git commit -m "feat: transporte con timecode grande y barra de estado"
```

---

### Task 19: Panel de atajos y teclado completo

**Files:**
- Create: `crates/app/src/ui/atajos.rs`
- Modify: `crates/app/src/main.rs`, `crates/app/src/ui/editor.rs`, `crates/app/src/textos.rs`

**Interfaces:**
- Produces: `pub fn mostrar(ctx: &Context, textos: &Textos, atajos: &MapaAtajos) -> bool` (abierto); el atajo `?`/`F1` se registra en `MapaAtajos::por_defecto()`.

- [ ] **Step 1: Test de contenido y cierre**

```rust
#[test]
fn el_panel_lista_todos_los_comandos_con_su_atajo_vigente() {
    let mut atajos = MapaAtajos::por_defecto();
    atajos.asignar(Comando::Dividir, Atajo { tecla: Key::D, ctrl: true, shift: false, alt: false }).unwrap();
    /* abrir el panel y comprobar que "Dividir" aparece junto a "Ctrl+D" */
}

#[test]
fn escape_cierra_el_panel() { /* … */ }
```

- [ ] **Step 2: Ejecutar y ver el fallo**

Run: `cargo test -p rod-app --test atajos`
Expected: FAIL.

- [ ] **Step 3: Implementar el panel y la navegación**

Capa de 560 con dos columnas agrupadas por `Seccion`, `Esc` y clic fuera cierran. Navegación por teclado en la lista de subtítulos (`↑`/`↓`/`Inicio`/`Fin`/`Enter`/`Tab`), anillo de foco visible en todo el kit, y `Ctrl+Tab` cambiando de panel.

- [ ] **Step 4: Tests y commit**

Run: `cargo test -p rod-app --test atajos`
Expected: PASS.

```bash
git add crates/app/src/ui/atajos.rs crates/app/src/main.rs crates/app/src/ui/editor.rs crates/app/src/textos.rs crates/app/tests/atajos.rs
git commit -m "feat: panel de atajos y navegacion completa por teclado"
```

---

### Task 20: Responsive, rendimiento y cierre

**Files:**
- Modify: `crates/app/examples/capturas.rs`, `crates/app/src/ui/{editor,inicio}.rs`, `docs/BITACORA.md`
- Test: `crates/app/tests/rendimiento.rs`

**Interfaces:**
- Consumes: todo lo anterior.

- [ ] **Step 1: Test de tamaño mínimo y máximo**

```rust
#[test]
fn a_1100_px_nada_se_solapa() {
    /* harness del editor a 1100x700 con el fixture; comprobar que la barra de
       estado y el transporte siguen dentro de la ventana y que no hay rects
       con ancho negativo (egui pinta un warning en ese caso) */
}
```

- [ ] **Step 2: Test de rendimiento con 543 líneas**

```rust
#[test]
fn el_pintado_del_editor_baja_de_ocho_milisegundos() {
    let mut app = editor_de_prueba();
    let ctx = egui::Context::default();
    rod_app::ui::tema::aplicar(&ctx);
    let inicio = std::time::Instant::now();
    for _ in 0..30 {
        ctx.run(Default::default(), |ctx| {
            rod_app::ui::editor::show(ctx, &mut app, wgpu::TextureFormat::Bgra8Unorm, std::sync::Arc::new(std::sync::Mutex::new(None)), &EN, &MapaAtajos::por_defecto(), &mut MedidasEditor::default());
        });
    }
    let media = inicio.elapsed() / 30;
    assert!(media < std::time::Duration::from_millis(8), "media {media:?}");
}
```

- [ ] **Step 3: Ejecutar y ajustar**

Run: `cargo test -p rod-app --test rendimiento`
Expected: PASS. Si no baja de 8 ms, el culpable está en el timeline: galerías recalculadas o búsqueda lineal por clip; arreglarlo ahí, no relajar el umbral.

- [ ] **Step 4: Capturas finales**

Run: `cargo run -p rod-app --example capturas -- finales`
Expected: `finales-inicio-{1100,1440,2560}.png`, `finales-paneles-editor-1440.png`
y `finales-inicio-es-1100.png` del arnés. Además, con `capturar.ps1`, captura
real del editor con video a 1100, 1440 y 2560 de ancho:
`finales-editor-{1100,1440,2560}.png`. Revisarlas una a una contra la spec §10
y el mockup; el monitor vacío solo es aceptable en las del arnés.

- [ ] **Step 5: Cierre**

`docs/BITACORA.md`: qué se desvió (Phosphor por Lucide, waveform si falló), qué queda pendiente (errores de `rod-core` sin traducir, «Ajustes» con más campos si aparecen). Actualizar la tabla de pendientes del proyecto si existe.

- [ ] **Step 6: Verificación final**

Run: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
Expected: todo PASS, sin warnings.

- [ ] **Step 7: Commit**

```bash
git add crates/app/examples/capturas.rs crates/app/src/ui docs/BITACORA.md docs/checklists/rediseno
git commit -m "feat: rediseno cerrado con capturas finales y prueba de rendimiento"
```
