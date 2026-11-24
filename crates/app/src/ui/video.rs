//! Puente entre el pipeline de video/libass y el renderer de egui. Los
//! `.wgsl` y la struct `Gpu` son los mismos que usaba iced: aquí solo cambia
//! quién entrega `device`/`queue` y el `RenderPass`.

use std::sync::{Arc, Mutex};

use eframe::egui_wgpu::{Callback, CallbackResources, CallbackTrait, ScreenDescriptor};
use egui::{PaintCallbackInfo, Rect, Ui};
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

use crate::video::{Frame, Gpu, geometria};

struct Dibujo {
    frame: Frame,
    rect: Rect,
    /// El pipeline se crea una vez con el `device` del renderer.
    gpu: Arc<Mutex<Option<Gpu>>>,
}

impl CallbackTrait for Dibujo {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        pantalla: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        _resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let escala = geometria::escala_letterbox(
            (self.frame.width, self.frame.height),
            (self.rect.width(), self.rect.height()),
        );
        let mut gpu = self
            .gpu
            .lock()
            .expect("el mutex del pipeline no está envenenado");
        let formato = eframe::egui_wgpu::preferred_framebuffer_format(&[])
            .expect("egui_wgpu devuelve un formato de framebuffer");
        let pipeline = gpu.get_or_insert_with(|| Gpu::nuevo(device, queue, formato));
        let _ = pantalla;
        pipeline.subir(device, queue, &self.frame, escala);
        Vec::new()
    }

    fn paint(
        &self,
        info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'static>,
        _resources: &CallbackResources,
    ) {
        let viewport = info.viewport_in_pixels();
        render_pass.set_viewport(
            viewport.left_px as f32,
            viewport.from_bottom_px as f32,
            viewport.width_px as f32,
            viewport.height_px as f32,
            0.0,
            1.0,
        );
        if let Some(gpu) = self
            .gpu
            .lock()
            .expect("el mutex del pipeline no está envenenado")
            .as_ref()
        {
            gpu.dibujar(render_pass);
        }
    }
}

pub fn pintar(ui: &mut Ui, rect: Rect, frame: Frame) {
    ui.painter().add(Callback::new_paint_callback(
        rect,
        Dibujo {
            frame,
            rect,
            gpu: Arc::new(Mutex::new(None)),
        },
    ));
}

#[cfg(test)]
mod tests {
    use crate::video::geometria::escala_letterbox;

    #[test]
    fn letterbox_en_rect_no_cuadrado() {
        let (x, y) = escala_letterbox((1920, 1080), (1000.0, 400.0));
        assert!((x - 16.0 / 9.0 / 2.5).abs() < 1e-6);
        assert!((y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cambio_de_resolucion_calcula_escala_distinta() {
        let primero = escala_letterbox((1920, 1080), (800.0, 600.0));
        let segundo = escala_letterbox((1280, 1024), (1000.0, 400.0));
        assert_ne!(primero, segundo);
    }
}
