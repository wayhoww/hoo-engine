use crate::{
    device::{
        graphics::{
            BBufferUsages, EBufferViewType, EVertexFormat, FAttachment, FBuffer, FBufferView,
            FDrawCommand, FGraphicsPass, FGraphicsPassEncoder, FMaterial, FTexture, FTextureView,
            FVertexEntry,
        },
        io::load_string,
    },
    utils::RcMut,
};

pub struct FCursorPass {
    pass: FGraphicsPass,
    draw_comand: FDrawCommand,
    material: FMaterial,
}

impl FCursorPass {
    pub fn new() -> Self {
        let mut mat = FMaterial::new(load_string("shaders/cursor.wgsl").unwrap());
        mat.enable_shader_profile("base".into());
        Self {
            pass: FGraphicsPass::default(),
            draw_comand: Self::create_fullscreen_draw_command(),
            material: mat,
        }
    }

    pub fn get_pass(&self, color: FTextureView) -> FGraphicsPass {
        let mut pass = self.pass.clone();
        pass.set_color_attachments(vec![FAttachment::new_append_to_view(color)]);
        return pass;
    }

    pub fn create_fullscreen_draw_command() -> FDrawCommand {
        let index_buffer = FBuffer::new_and_manage(BBufferUsages::Index);
        let vertex_buffer_position = FBuffer::new_and_manage(BBufferUsages::Vertex);
        let vertex_buffer_uv = FBuffer::new_and_manage(BBufferUsages::Vertex);

        index_buffer
            .borrow_mut()
            .update_by_array(&[0u32, 2, 1, 3, 2, 0]);
        vertex_buffer_position.borrow_mut().update_by_array(&[
            -1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 0.0f32,
        ]);
        vertex_buffer_uv
            .borrow_mut()
            .update_by_array(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0f32]);

        let entry_position = FVertexEntry::new_soa_entry(
            0,
            FBufferView::new_with_type(vertex_buffer_position, EBufferViewType::Vertex),
            EVertexFormat::Float32x3,
        );
        let entry_uv = FVertexEntry::new_soa_entry(
            1,
            FBufferView::new_with_type(vertex_buffer_uv, EBufferViewType::Vertex),
            EVertexFormat::Float32x2,
        );
        let entries = vec![entry_position, entry_uv];

        FDrawCommand::new(
            entries,
            FBufferView::new_with_type(index_buffer, EBufferViewType::Index),
            6,
            FBufferView::default(),
            FBufferView::default(),
        )
    }

    pub fn encode_pass(&self, pass_encoder: &mut FGraphicsPassEncoder) {
        pass_encoder.setup_pipeline(
            &self.draw_comand.get_vertex_buffers(),
            &self.material.get_shader_module("base").unwrap(),
        );
        pass_encoder.draw(&self.draw_comand);

        // pass_encoder.draw()
    }
}
