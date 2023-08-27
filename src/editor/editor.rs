use crate::{
    device::graphics::{FEguiGraphicsContext, FTexture, FTextureView},
    hoo_engine,
    utils::RcMut,
};

pub struct FEditor {
    pub main_viewport_texture: Option<RcMut<FTexture>>,
}

impl FEditor {
    pub fn new() -> Self {
        Self {
            main_viewport_texture: None,
        }
    }

    pub fn draw(&self, ctx: &egui::Context, mut gctx: FEguiGraphicsContext) {
        egui::Window::new("Editor").show(ctx, |ui| {
            ui.label("Hello World!");
        });

        egui::Window::new("Viewport")
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(tex) = self.main_viewport_texture.as_ref() {
                    gctx.image(ui, tex);
                } else {
                    ui.label("No viewport texture");
                }
            });
    }
}
