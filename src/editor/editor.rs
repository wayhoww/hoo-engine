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
                    let image = gctx.image(ui, tex);

                    if let Some(pointer_pos) = image.hover_pos() {
                        let pointer_pos = (pointer_pos.x - image.rect.left(), image.rect.bottom() - pointer_pos.y);
                        let pointer_uv = (pointer_pos.0 / image.rect.width(), pointer_pos.1 / image.rect.height());
                        // println!("hover_pos: {:?}", pointer_uv);
                    }
                } else {
                    ui.label("No viewport texture");
                }
            });
    }
}
