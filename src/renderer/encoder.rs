use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use web_sys::*;

use crate::{hoo_log, renderer::utils::jsarray, utils::types::RcMut, HooEngineRef};

use super::resource::{FBufferView, FPass, FShaderModule, FTexture};

use strum::*;
use strum_macros::*;

struct FPipeline {
    // descriptor
}

impl FPipeline {
    fn new() -> Self {
        Self {}
    }

    // ensure pass exists in encoder!
    fn create_device_resource_with_pass(
        &mut self,
        encoder: &mut FWebGPUEncoder,
        shader_module: &RcMut<FShaderModule>,
    ) -> web_sys::GpuRenderPipeline {
        let logical_shader_module = shader_module.borrow();
        let shader_module = logical_shader_module.get_device_module().unwrap();

        let vertex_buffer_layout = web_sys::GpuVertexBufferLayout::new(
            32.0,
            &jsarray(&[
                web_sys::GpuVertexAttribute::new(web_sys::GpuVertexFormat::Float32x3, 0.0, 0),
                web_sys::GpuVertexAttribute::new(web_sys::GpuVertexFormat::Float32x3, 12.0, 1),
                web_sys::GpuVertexAttribute::new(web_sys::GpuVertexFormat::Float32x2, 24.0, 2),
            ]),
        );

        let mut vertex_state = web_sys::GpuVertexState::new(
            logical_shader_module.get_vertex_stage_entry().unwrap(),
            shader_module,
        );

        vertex_state.buffers(&jsarray([&vertex_buffer_layout].as_slice()));

        let pass = encoder.current_pass.as_ref().unwrap();

        let color_attachments: Vec<JsValue> = pass
            .get_color_attachments()
            .into_iter()
            .map(|view| {
                let texref = view.texture_view.get_texture();
                let tex = texref.borrow();
                JsValue::from(web_sys::GpuColorTargetState::new(tex.get_format().into()))
            })
            .collect();

        let fragment_state = web_sys::GpuFragmentState::new(
            logical_shader_module.get_fragment_stage_entry().unwrap(),
            &shader_module,
            &jsarray(&color_attachments),
        );

        let pipeline_layout = encoder.get_device().create_pipeline_layout(
            &web_sys::GpuPipelineLayoutDescriptor::new(&jsarray(encoder.get_bind_group_layouts())),
        );

        let mut descriptor =
            web_sys::GpuRenderPipelineDescriptor::new(&pipeline_layout, &vertex_state);
        descriptor.fragment(&fragment_state);

        if let Some(view) = pass.get_depth_stencil_attachment() {
            let texref = view.get_texture();
            let tex = texref.borrow();
            descriptor.depth_stencil(
                GpuDepthStencilState::new(tex.get_format().into())
                    .depth_compare(GpuCompareFunction::LessEqual)
                    .depth_write_enabled(true),
            );
        }

        encoder.get_device().create_render_pipeline(&descriptor)
    }
}

// 顺序和 shader 保持一致
#[derive(EnumCount)]
#[repr(u32)]
enum EUniformBufferType {
    Material,
    DrawCall,
    Pass,
    Task,
    Global,
}

pub struct FWebGPUEncoder {
    device: GpuDevice,
    context: GpuCanvasContext,
    swapchain_texture: RcMut<FTexture>,

    encoder: Option<GpuCommandEncoder>,
    pass_encoder: Option<GpuRenderPassEncoder>,

    bind_group_layouts: Vec<GpuBindGroupLayout>,

    uniform_buffer_view_global: Option<FBufferView>,
    uniform_buffer_view_task: Option<FBufferView>,

    // bind_group_global: Option<GpuBindGroup>,
    // bind_group_task: Option<GpuBindGroup>,
    // bind_group_pass: Option<GpuBindGroup>,
    current_pass: Option<FPass>,
}

impl FWebGPUEncoder {
    // getter
    pub fn get_device(&self) -> &GpuDevice {
        &self.device
    }

    pub fn get_context(&self) -> &GpuCanvasContext {
        &self.context
    }

    pub fn get_encoder(&self) -> &Option<GpuCommandEncoder> {
        &self.encoder
    }

    pub fn get_bind_group_layouts(&self) -> &Vec<GpuBindGroupLayout> {
        &self.bind_group_layouts
    }

    // impl

    pub fn new(h: HooEngineRef, context: GpuCanvasContext) -> Self {
        futures::executor::block_on(Self::new_async(h, context))
    }

    pub async fn new_async<'a>(h: HooEngineRef<'a>, context: GpuCanvasContext) -> Self {
        let navigator = window().unwrap().navigator();
        let gpu = navigator.gpu();
        let adapter = JsFuture::from(gpu.request_adapter())
            .await
            .unwrap()
            .dyn_into::<GpuAdapter>()
            .unwrap();
        let device = JsFuture::from(adapter.request_device())
            .await
            .unwrap()
            .dyn_into::<GpuDevice>()
            .unwrap();
        let swapchain_format = GpuTextureFormat::Bgra8unorm;

        context.configure(&GpuCanvasConfiguration::new(&device, swapchain_format));
        let swapchain_texture = FTexture::new_swapchain_texture_and_manage(h, &context);

        let bind_group_layouts = Self::make_bind_group_layouts(&device);

        Self {
            device: device,
            context: context,
            swapchain_texture,

            encoder: None,
            pass_encoder: None,

            bind_group_layouts: bind_group_layouts,

            uniform_buffer_view_global: None,
            uniform_buffer_view_task: None,

            // bind_group_global: None,
            // bind_group_task: None,
            // bind_group_pass: None,
            current_pass: None,
        }
    }

    pub fn get_swapchain_texture(&self) -> RcMut<FTexture> {
        return self.swapchain_texture.clone();
    }

    pub fn get_swapchain_size(&self) -> (u32, u32) {
        let context = &self.context;
        let swapchain_size = (
            context.get_current_texture().width(),
            context.get_current_texture().height(),
        );
        swapchain_size
    }

    // 所有 pipeline 的 bind group layout 都是一致的, 若干个 cbuffer + 若干个贴图
    // cbuffer: MaterialUniform, DrawCallUniform, PassUniform, TaskUniform, GlobalUniform
    // 贴图：固定数量

    // DrawCallUniform: 在 DrawCommand 中提供，大小可变
    // PassUniform: 在 Pass 中提供，大小可变
    // TaskUniform: 暂时没想好如何做抽象，先通过 SetTaskUniform 设置。类似于 Viewport 的概念，表示一个完整的渲染管线。
    // GlobalUniform: 通过 SetGlobalUniformBuffer 设置，大小固定
    fn make_bind_group_layouts(device: &GpuDevice) -> Vec<GpuBindGroupLayout> {
        let mut bind_group_layout_entries = vec![];
        for i in 0..EUniformBufferType::COUNT {
            let mut entry = GpuBindGroupLayoutEntry::new(
                i as u32,
                gpu_shader_stage::VERTEX | gpu_shader_stage::FRAGMENT | gpu_shader_stage::COMPUTE,
            );
            entry.buffer(&GpuBufferBindingLayout::new().type_(GpuBufferBindingType::Uniform));
            bind_group_layout_entries.push(entry);
        }
        let out = device.create_bind_group_layout(
            &GpuBindGroupLayoutDescriptor::new(&jsarray(&bind_group_layout_entries))
                .label("BindGroup-0"),
        );
        return vec![out];

        // let mut bind_group_layout_entry_uniform_template = GpuBindGroupLayoutEntry::new(
        //     0,
        //     gpu_shader_stage::VERTEX | gpu_shader_stage::FRAGMENT | gpu_shader_stage::COMPUTE,
        // );
        // bind_group_layout_entry_uniform_template
        //     .buffer(&GpuBufferBindingLayout::new().type_(GpuBufferBindingType::Uniform));

        // let mut out = Vec::new();

        // // Global, Task, Pass, DrawCall, Material
        // for _ in EUniformBufferType::VARIANTS {
        //     let mut bind_group_layout_entry_array = Vec::new();

        //     // uniform buffer
        //     let bind_group_layout_entry_uniform = bind_group_layout_entry_uniform_template.clone();
        //     bind_group_layout_entry_array.push(bind_group_layout_entry_uniform);

        //     // textures, etc.

        //     let bind_group_layout =
        //         device.create_bind_group_layout(&GpuBindGroupLayoutDescriptor::new(&jsarray(
        //             bind_group_layout_entry_array.as_slice(),
        //         )));
        //     out.push(bind_group_layout);
        // }

        // out
    }

    fn create_bind_group_entry_of_buffer(
        &self,
        binding: u32,
        buffer: &FBufferView,
    ) -> GpuBindGroupEntry {
        GpuBindGroupEntry::new(
            binding,
            &web_sys::GpuBufferBinding::new(buffer.get_buffer().borrow().get_device_buffer())
                .offset(buffer.get_offset() as f64)
                .size(buffer.get_size() as f64),
        )
    }

    pub fn set_global_uniform_buffer_view(&mut self, buffer: FBufferView) {
        self.uniform_buffer_view_global = Some(buffer);
        // self.update_bind_group_global();
    }

    pub fn set_task_uniform_buffer_view(&mut self, buffer: FBufferView) {
        self.uniform_buffer_view_task = Some(buffer);
        // self.update_bind_group_task();
    }

    // fn update_bind_group_global(&mut self) {
    //     let buffer = self.uniform_buffer_view_global.as_ref().unwrap();

    //     let bind_group_entries = [self.create_bind_group_entry_of_buffer(0, buffer)];

    //     let bind_group = self
    //         .device
    //         .create_bind_group(&web_sys::GpuBindGroupDescriptor::new(
    //             &jsarray(bind_group_entries.as_slice()),
    //             &self.bind_group_layouts[0],
    //         ));

    //     self.bind_group_global = Some(bind_group);
    // }

    // fn update_bind_group_task(&mut self) {
    //     let buffer = self.uniform_buffer_view_task.as_ref().unwrap();

    //     let bind_group_entries = [self.create_bind_group_entry_of_buffer(0, buffer)];

    //     let bind_group = self
    //         .device
    //         .create_bind_group(&web_sys::GpuBindGroupDescriptor::new(
    //             &jsarray(bind_group_entries.as_slice()),
    //             &self.bind_group_layouts[1],
    //         ));

    //     self.bind_group_task = Some(bind_group);
    // }

    pub fn begin_frame(&mut self) {
        debug_assert!(self.encoder.is_none());
        self.encoder = Some(self.device.create_command_encoder());
        self.swapchain_texture
            .borrow_mut()
            .update_swapchain_texture(&self.context);
    }

    pub fn end_frame(&mut self) {
        self.swapchain_texture
            .borrow_mut()
            .clear_swapchain_texture();

        let encoder = self.encoder.take().unwrap();
        self.device
            .queue()
            .submit(&jsarray(&[encoder.finish()].as_slice()));
    }

    // fn create_pass_bind_group(&mut self, pass: &FPass) {
    //     debug_assert!(self.bind_group_pass.is_none());

    //     let bind_group = self.device.create_bind_group(&GpuBindGroupDescriptor::new(
    //         &jsarray(
    //             [self.create_bind_group_entry_of_buffer(0, pass.get_uniform_buffer_view())]
    //                 .as_slice(),
    //         ),
    //         &self.bind_group_layouts[2],
    //     ));
    //     self.bind_group_pass = Some(bind_group);
    // }

    pub fn begin_render_pass(&mut self, render_pass: &FPass) {
        debug_assert!(self.pass_encoder.is_none());
        debug_assert!(self.current_pass.is_none());

        // self.create_pass_bind_group(render_pass);
        self.current_pass = Some(render_pass.clone());

        let encoder = self.encoder.as_ref().unwrap();

        let color_attachments: Vec<_> = render_pass
            .get_color_attachments()
            .into_iter()
            .map(|x| {
                let color_attachment = GpuRenderPassColorAttachment::new(
                    x.load_op.clone().into(),
                    x.store_op.clone().into(),
                    &x.texture_view.get_device_texture_view(),
                );
                JsValue::from(color_attachment)
            })
            .collect();

        let mut render_pass_descriptor = GpuRenderPassDescriptor::new(&jsarray(&color_attachments));

        if let Some(dsv) = render_pass.get_depth_stencil_attachment() {
            let mut desc = GpuRenderPassDepthStencilAttachment::new(&dsv.get_device_texture_view());

            desc.depth_load_op(GpuLoadOp::Clear);
            desc.depth_store_op(GpuStoreOp::Store);
            desc.depth_clear_value(1.0);

            desc.stencil_load_op(GpuLoadOp::Clear);
            desc.stencil_store_op(GpuStoreOp::Store);
            desc.stencil_clear_value(0);
            desc.stencil_read_only(false);

            render_pass_descriptor.depth_stencil_attachment(&desc);
        }

        let depth_stencil_attachment: Option<GpuRenderPassDepthStencilAttachment> = None;
        if let Some(depth_stencil_attachment) = depth_stencil_attachment {
            render_pass_descriptor.depth_stencil_attachment(&depth_stencil_attachment);
        }

        let pass = encoder.begin_render_pass(&render_pass_descriptor);
        self.pass_encoder = Some(pass);
    }

    pub fn end_render_pass(&mut self) {
        self.pass_encoder.take().unwrap().end();
        self.current_pass.take().unwrap();
        // self.bind_group_pass.take().unwrap();
    }

    pub fn setup_pipeline(&mut self, shader_module: &RcMut<FShaderModule>) {
        let mut pipeline = FPipeline::new();
        let device_pipeline = pipeline.create_device_resource_with_pass(self, shader_module);
        let pass_encoder = self.pass_encoder.as_ref().unwrap();
        pass_encoder.set_pipeline(&device_pipeline);
    }

    pub fn draw(&mut self, draw_command: &crate::renderer::resource::FDrawCommand) {
        let pass_encoder = self.pass_encoder.as_ref().unwrap();
        // let current_pass = &self.current_pass;

        let vertex_buffer_view = draw_command.get_vertex_buffer_view();
        let index_buffer_view = draw_command.get_index_buffer_view();

        pass_encoder.set_vertex_buffer_with_u32_and_u32(
            0,
            vertex_buffer_view.get_buffer().borrow().get_device_buffer(),
            vertex_buffer_view.get_offset(),
            vertex_buffer_view.get_size(),
        );

        pass_encoder.set_index_buffer_with_u32_and_u32(
            index_buffer_view.get_buffer().borrow().get_device_buffer(),
            GpuIndexFormat::Uint32,
            index_buffer_view.get_offset(),
            index_buffer_view.get_size(),
        );

        let bindgroup_0 = self.device.create_bind_group(&GpuBindGroupDescriptor::new(
            &jsarray(
                [
                    self.create_bind_group_entry_of_buffer(
                        EUniformBufferType::Material as u32,
                        draw_command.get_material_view(),
                    ),
                    self.create_bind_group_entry_of_buffer(
                        EUniformBufferType::DrawCall as u32,
                        draw_command.get_drawcall_view(),
                    ),
                    self.create_bind_group_entry_of_buffer(
                        EUniformBufferType::Pass as u32,
                        self.current_pass
                            .as_ref()
                            .unwrap()
                            .get_uniform_buffer_view(),
                    ),
                    self.create_bind_group_entry_of_buffer(
                        EUniformBufferType::Task as u32,
                        self.uniform_buffer_view_task.as_ref().unwrap(),
                    ),
                    self.create_bind_group_entry_of_buffer(
                        EUniformBufferType::Global as u32,
                        self.uniform_buffer_view_global.as_ref().unwrap(),
                    ),
                ]
                .as_slice(),
            ),
            &self.bind_group_layouts[0],
        ));

        pass_encoder.set_bind_group(0, &bindgroup_0);

        pass_encoder.draw_indexed(draw_command.get_index_count());
    }
}
