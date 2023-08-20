pub struct FEditor {}

impl FEditor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, ctx: &egui::Context) {
        egui::Window::new("Editor").show(ctx, |ui| {
            ui.label("Hello World!");
        });
        
        egui::Window::new("Viewport").show(ctx, |ui| {
            
            // ui.image(egui::TextureId::User(0), [512.0, 512.0]);
        });
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        // self.angle += response.drag_delta().x * 0.01;

        // Clone locals so we can move them into the paint callback:
        // let angle = self.angle;
        // let rotating_triangle = self.rotating_triangle.clone();

        // let callback = egui::PaintCallback {
        //     rect,
        //     // callback: std::sync::Arc::new(egui_wgpu::CallbackFn::paint()),
        // };
        ui.painter().add(callback);
    }
}
