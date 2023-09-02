use crate::{
    device::graphics::{FEguiGraphicsContext, FTexture, FTextureView},
    hoo_engine,
    utils::RcMut,
};

pub struct FEditorState {
    pub overlay_mode: bool, // write_only
    pub main_viewport_cursor_position: Option<(f32, f32)>,
}

impl FEditorState {
    pub fn new() -> Self {
        Self {
            overlay_mode: false,
            main_viewport_cursor_position: None,
        }
    }
}

pub struct FEditor {
    pub main_viewport_texture: Option<RcMut<FTexture>>,
    pub state: FEditorState,
}

pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    // Widget code can be broken up in four steps:
    //  1. Decide a size for the widget
    //  2. Allocate space for it
    //  3. Handle interactions with the widget (if any)
    //  4. Paint the widget

    // 1. Deciding widget size:
    // You can query the `ui` how much space is available,
    // but in this example we have a fixed size widget based on the height of a standard button:
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);

    // 2. Allocating space:
    // This is where we get a region of the screen assigned.
    // We also tell the Ui to sense clicks in the allocated region.
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // 3. Interact: Time to check for clicks!
    if response.clicked() {
        *on = !*on;
        response.mark_changed(); // report back that the value changed
    }

    // Attach some meta-data to the response which can be used by screen readers:
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    // 4. Paint!
    // Make sure we need to paint:
    if ui.is_rect_visible(rect) {
        // Let's ask for a simple animation from egui.
        // egui keeps track of changes in the boolean associated with the id and
        // returns an animated value in the 0-1 range for how much "on" we are.
        let how_on = ui.ctx().animate_bool(response.id, *on);
        // We will follow the current style by asking
        // "how should something that is being interacted with be painted?".
        // This will, for instance, give us different colors when the widget is hovered or clicked.
        let visuals = ui.style().interact_selectable(&response, *on);
        // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        // Paint the circle, animating it from left to right with `how_on`:
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    // All done! Return the interaction response so the user can check what happened
    // (hovered, clicked, ...) and maybe show a tooltip:
    response
}

impl FEditor {
    pub fn new() -> Self {
        Self {
            main_viewport_texture: None,
            state: FEditorState::new(),
        }
    }

    pub fn get_state(&self) -> &FEditorState {
        &self.state
    }

    pub fn draw(&mut self, ctx: &egui::Context, mut gctx: FEguiGraphicsContext) {
        // egui::Image::new(gctx.register_texture_online(
        //     self.main_viewport_texture.as_ref().unwrap(),
        // ), egui::vec2(512.0, 512.0)).paint_at(ui);

        if self.state.overlay_mode {
            self.state.main_viewport_cursor_position = ctx.pointer_hover_pos().map(|pos| {
                let rect = ctx.available_rect();
                self.state.main_viewport_cursor_position = Some((pos.x, pos.y));
                ((pos.x - rect.left()) * gctx.get_scale_factor(), (pos.y - rect.top()) * gctx.get_scale_factor())
            });
        }

        egui::SidePanel::new(egui::panel::Side::Left, egui::Id::new("left-panel")).show(
            ctx,
            |ui| {
                ui.heading("Left Panel");
                ui.label("This is the left panel");
            },
        );

        egui::Window::new("Editor Setting").show(ctx, |ui| {
            ui.label("Overlay Mode");
            toggle_ui(ui, &mut self.state.overlay_mode);
        });

        if !self.state.overlay_mode {
            egui::panel::CentralPanel::default().show(ctx, |ui| {
                if let Some(tex) = self.main_viewport_texture.as_ref() {
                    let image = gctx.image(ui, tex);
                    let pointer_pos = image
                        .hover_pos()
                        .map(|pos| ((pos.x - image.rect.left()) * gctx.get_scale_factor(), (pos.y - image.rect.top()) * gctx.get_scale_factor()));
                    self.state.main_viewport_cursor_position = pointer_pos;
                } else {
                    ui.label("No viewport texture");
                }
            });
        }

        // egui::Window::new("Viewport")
        //     .resizable(true)
        //     .show(ctx, |ui| {
        //         if let Some(tex) = self.main_viewport_texture.as_ref() {
        //             let image = gctx.image(ui, tex);

        //             if let Some(pointer_pos) = image.hover_pos() {
        //                 let pointer_pos = (
        //                     pointer_pos.x - image.rect.left(),
        //                     image.rect.bottom() - pointer_pos.y,
        //                 );
        //                 let pointer_uv = (
        //                     pointer_pos.0 / image.rect.width(),
        //                     pointer_pos.1 / image.rect.height(),
        //                 );
        //                 // println!("hover_pos: {:?}", pointer_uv);
        //             }
        //         } else {
        //             ui.label("No viewport texture");
        //         }
        //     });
    }
}
