use bitflags::bitflags;

use crate::{
    bundle, editor::importer::load_gltf_from_slice, hoo_log, rcmut, utils::*, HooEngineRef,
    HooEngineWeak,
};

use super::{encoder::*, resource::*};

use nalgebra_glm as glm;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Flags: u32 {
        const A = 0b00000001;
        const B = 0b00000010;
        const C = 0b00000100;
        const ABC = Self::A.bits() | Self::B.bits() | Self::C.bits();
    }
}

pub struct FGraphicsPipeline {
    hoo_engine: HooEngineWeak,

    pass1: FPass,
    pass2: FPass,

    uniform_buffer: RcMut<FBuffer>,
    uniform_view: FBufferView,
    model: FModel,

    render_object1: FRenderObject,
    render_object2: FRenderObject,

    count: u32,
}

// 实现基本功能：
// 多 pass 支持
// 不同 pass 收集不同的 flags

impl FGraphicsPipeline {
    pub async fn new_async<'a>(h: HooEngineRef<'a>, encoder: &FWebGPUEncoder) -> Self {
        // material
        let shader_code = get_text_from_url("/resources/main.wgsl").await.unwrap();
        let material = rcmut!(FMaterial::new(h, shader_code));
        material.borrow_mut().enable_pass_variant("base".into());
        material
            .borrow_mut()
            .enable_pass_variant("depthOnly".into());

        // mesh
        let load_result = load_gltf_from_slice(bundle::gltf_cube());
        let mesh = rcmut!(FMesh::from_file_resource(
            h,
            &load_result.unwrap()[0].sub_meshes[0]
        ));

        // model
        let model = FModel::new(&h, mesh, material.clone());
        let render_object1 = FRenderObject::new(h, model.clone());
        let render_object2 = FRenderObject::new(h, model.clone());

        // faked
        let default_uniform_buffer = FBuffer::new_and_manage(h, BBufferUsages::Uniform);
        default_uniform_buffer.borrow_mut().resize(1024);
        let uniform_view = FBufferView::new_uniform(default_uniform_buffer.clone());

        // pass
        let swapchain_image = encoder.get_swapchain_texture();
        let depth_texture =
            FTexture::new_and_manage(h, EValueFormat::Depth24Stencil8, BTextureUsages::Attachment);
        depth_texture
            .borrow_mut()
            .set_size(swapchain_image.borrow_mut().get_size());
        let depth_texture_view = FTextureView::new(depth_texture.clone());

        let mut pass1 = FPass::new(uniform_view.clone());
        let mut pass2 = FPass::new(uniform_view.clone());
        pass1.set_depth_stencil_attachment(depth_texture_view.clone());
        pass2.set_depth_stencil_attachment(depth_texture_view.clone());

        Self {
            hoo_engine: h.clone(),
            pass1,
            pass2,
            uniform_buffer: default_uniform_buffer,
            uniform_view,
            model,
            render_object1,
            render_object2,
            count: 0,
        }
    }

    pub fn draw(&mut self, encoder: &mut FWebGPUEncoder) {
        // before draw

        self.count += 1;
        let mat_model: glm::Mat4x4 =
            glm::rotation(self.count as f32 * 0.01, &glm::vec3(0.0, 1.0, 0.0));

        let mat_model_biased = glm::translation(&glm::vec3(3.0, 0.0, 0.0)) * mat_model;

        let mat_view = glm::look_at(
            &glm::vec3(3.0, 3.0, 3.0),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );

        let size = encoder.get_swapchain_size();
        let mat_proj = glm::Mat4x4::new_perspective(
            size.0 as f32 / size.1 as f32,
            glm::pi::<f32>() * 0.6,
            0.1,
            100.0,
        );

        // let mut uniform_buffer_data = Vec::new();
        // uniform_buffer_data.extend_from_slice(mat_model.as_slice());
        // uniform_buffer_data.extend_from_slice(mat_view.as_slice());
        // uniform_buffer_data.extend_from_slice(mat_proj.as_slice());

        // self.uniform_buffer
        //     .borrow_mut()
        //     .update_by_array(&uniform_buffer_data);
        // self.uniform_buffer.borrow_mut().resize(1024);

        // draw

        self.hoo_engine
            .upgrade()
            .unwrap()
            .borrow()
            .get_resources()
            .prepare_gpu_resources(encoder);

        // TODO: should be updated somewhere else
        encoder.set_global_uniform_buffer_view(self.uniform_view.clone());

        encoder.begin_frame();

        let swapchain_image = encoder.get_swapchain_texture();
        let swapchain_image_view = FTextureView::new(swapchain_image.clone());

        self.pass1.set_color_attachments(vec![FColorAttachment::new(
            swapchain_image_view.clone(),
            ELoadOp::Clear,
            EStoreOp::Store,
        )]);

        self.pass2.set_color_attachments(vec![FColorAttachment::new(
            swapchain_image_view.clone(),
            ELoadOp::Load,
            EStoreOp::Store,
        )]);

        encoder.set_task_uniform_buffer_view(self.uniform_view.clone());

        self.render_object1
            .set_transform_model(mat_model.clone())
            .set_transform_view(mat_view.clone())
            .set_transform_projection(mat_proj.clone())
            .update_uniform_buffer();

        self.render_object2
            .set_transform_model(mat_model_biased.clone())
            .set_transform_view(mat_view.clone())
            .set_transform_projection(mat_proj.clone())
            .update_uniform_buffer();

        encoder.begin_render_pass(&self.pass1);
        self.render_object1.encode(encoder, "base");
        encoder.end_render_pass();

        encoder.begin_render_pass(&self.pass2);
        self.render_object2.encode(encoder, "base");
        encoder.end_render_pass();

        encoder.end_frame();
    }
}
