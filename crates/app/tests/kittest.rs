//! Comprobación del arnés de capturas: egui se renderiza a PNG sin abrir ventana.

#[test]
fn kittest_renderiza_un_rectangulo() {
    let mut harness = egui_kittest::Harness::builder()
        .wgpu()
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
