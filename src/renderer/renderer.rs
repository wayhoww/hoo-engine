use web_sys::GpuCanvasContext;

use crate::utils::get_text_from_url;

use crate::*;

use super::encoder::*;
use super::pipeline::FGraphicsPipeline;
use super::resource::*;

use nalgebra_glm as glm;

pub struct Renderer {
    hoo_engine: HooEngineWeak,

    // // resources
    graphics_encoder: FWebGPUEncoder,
    graphics_pipeline: Option<FGraphicsPipeline>,
    // uniform_buffer: FBuffer,
    // model: Option<FModel>,
    // pass: Option<FPass>,
}

impl Renderer {
    pub async fn new_async<'a>(h: HooEngineRef<'a>, canvas_ctx: GpuCanvasContext) -> Self {
        let graphics_encoder = FWebGPUEncoder::new_async(h, canvas_ctx).await;

        Self {
            hoo_engine: h.clone(),
            graphics_encoder,
            graphics_pipeline: None,
        }
    }

    pub fn new(h: HooEngineRef, canvas_ctx: GpuCanvasContext) -> Self {
        futures::executor::block_on(Self::new_async(h, canvas_ctx))
    }

    pub async fn initialize_test_resources(&mut self) {
        let graphics_pipeline =
            FGraphicsPipeline::new_async(&self.hoo_engine, &self.graphics_encoder).await;
        self.graphics_pipeline = Some(graphics_pipeline);

        // let load_result = load_gltf_from_slice(bundle::gltf_cube());
        // // hoo_log!("{:#?}", load_result);

        // let shader_code = get_text_from_url("/resources/main.wgsl").await.unwrap();
        // let material = rcmut!(FMaterial::new(&self.hoo_engine, shader_code));
        // material.borrow_mut().enable_pass_variant("base".into());

        // let mesh = rcmut!(FMesh::from_file_resource(
        //     &self.hoo_engine,
        //     &load_result.unwrap()[0].sub_meshes[0]
        // ));

        // let model = FModel::new(&self.hoo_engine, mesh, material.clone());

        // let uniform_buffer = FBuffer::new_and_manage(&self.hoo_engine, BBufferUsages::Uniform);

        // let mat_model: glm::Mat4x4 = glm::identity();
        // let mat_view = glm::look_at(
        //     &glm::vec3(3.0, 3.0, 3.0),
        //     &glm::vec3(0.0, 0.0, 0.0),
        //     &glm::vec3(0.0, 1.0, 0.0),
        // );

        // let size = self.graphics_encoder.get_swapchain_size();
        // let mat_proj = glm::Mat4x4::new_perspective(
        //     size.0 as f32 / size.1 as f32,
        //     glm::pi::<f32>() * 0.6,
        //     0.1,
        //     100.0,
        // );

        // let mut uniform_buffer_data = Vec::new();
        // uniform_buffer_data.extend_from_slice(mat_model.as_slice());
        // uniform_buffer_data.extend_from_slice(mat_view.as_slice());
        // uniform_buffer_data.extend_from_slice(mat_proj.as_slice());

        // uniform_buffer
        //     .borrow_mut()
        //     .update_by_array(&uniform_buffer_data);

        // // TODO: 知识点补齐 ~ 1   View & Proj Matrix

        // // hoo_log!("{:#?}", mat_view);
        // // for z in 0..10 {
        // //     let vec = glm::vec4(0.0, 0.0, z as f32, 1.0);
        // //     let view = mat_view * vec;
        // //     hoo_log!("vec: {}, view: {}", vec, view);
        // // }

        // // for z in 0..10 {
        // //     let vec = glm::vec4(0.0, 0.0, 0.1f32.max(z as f32), 1.0);
        // //     let proj = mat_proj * vec;
        // //     hoo_log!(
        // //         "z: {}, proj: {:#?}, proj.w/proj.z: {}",
        // //         z,
        // //         proj,
        // //         proj.w / proj.z
        // //     );
        // // }

        // let uniform_buffer_size = uniform_buffer.borrow_mut().size();
        // let uniform_buffer_view = FBufferView::new(
        //     uniform_buffer.clone(),
        //     0,
        //     uniform_buffer_size,
        //     EBufferViewType::Uniform,
        // );

        // let depth_stencil_texture = FTexture::new_and_manage(
        //     &self.hoo_engine,
        //     EValueFormat::Depth24Stencil8,
        //     BTextureUsages::Attachment,
        // );

        // depth_stencil_texture
        //     .borrow_mut()
        //     .set_size(self.graphics_encoder.get_swapchain_size());

        // let mut pass = FPass::new(uniform_buffer_view.clone());
        // pass.set_color_attachment(
        //     0,
        //     FTextureView::new(self.graphics_encoder.get_swapchain_texture()),
        // );
        // pass.set_depth_stencil_attachment(FTextureView::new(depth_stencil_texture.clone()));

        // self.hoo_engine
        //     .upgrade()
        //     .unwrap()
        //     .borrow()
        //     .get_resources()
        //     .prepare_gpu_resources(&mut self.graphics_encoder);

        // self.graphics_encoder
        //     .set_global_uniform_buffer_view(uniform_buffer_view.clone());
        // self.graphics_encoder
        //     .set_task_uniform_buffer_view(uniform_buffer_view.clone());

        // self.pass = Some(pass);
        // self.model = Some(model);
    }

    pub fn next_frame(&mut self) {
        self.graphics_pipeline
            .as_mut()
            .unwrap()
            .draw(&mut self.graphics_encoder);

        // encoder.begin_frame();

        // encoder.begin_render_pass(self.pass.as_ref().unwrap());

        // self.model.as_ref().unwrap().encode(encoder);

        // encoder.end_render_pass();

        // encoder.end_frame();
    }
}
