use std::cell::{Ref, RefCell};
use std::convert::TryFrom;
use std::ops::Deref;

use crate::utils::*;
use crate::*;

use super::resource::*;

use strum::*;
use strum_macros::*;

use egui_wgpu::wgpu;

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
        pass_encoder: &mut FPassEncoder,
        vertex_entries: &[FVertexEntry],
        shader_module: &RcMut<FShaderModule>,
    ) -> wgpu::RenderPipeline {
        let logical_shader_module = shader_module.borrow();
        let shader_module = logical_shader_module.get_device_module().unwrap();

        let attributes = vertex_entries
            .iter()
            .map(|entry| {
                [wgpu::VertexAttribute {
                    format: entry.format.into(),
                    offset: entry.offset,
                    shader_location: entry.location,
                }]
            })
            .collect::<Vec<_>>();

        let entries = (vertex_entries.iter().zip(attributes.iter()))
            .map(|(entry, attr)| wgpu::VertexBufferLayout {
                array_stride: entry.stride,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: attr,
            })
            .collect::<Vec<_>>();

        let vertex_state = wgpu::VertexState {
            module: shader_module,
            entry_point: logical_shader_module.get_vertex_stage_entry().unwrap(),
            buffers: &entries,
        };

        let color_attachments: Vec<_> = pass_encoder
            .pass
            .get_color_attachments()
            .iter()
            .map(|view| {
                let attachment = wgpu::ColorTargetState {
                    format: view.texture_view.get_format(pass_encoder.encoder).into(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                };
                Some(attachment)
            })
            .collect();

        let fragment_state = wgpu::FragmentState {
            module: shader_module,
            entry_point: logical_shader_module.get_fragment_stage_entry().unwrap(),
            targets: &color_attachments,
        };

        let depth_stencil_state = pass_encoder
            .pass
            .get_depth_stencil_attachment()
            .iter()
            .map(|view| {
                let texref = view.texture_view.get_texture();
                let _tex = texref.borrow();

                wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }
            })
            .collect::<Vec<_>>()
            .pop();

        let pipeline_layout = pass_encoder.encoder.get_device().create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &pass_encoder
                    .encoder
                    .get_bind_group_layouts()
                    .iter()
                    .collect::<Vec<&wgpu::BindGroupLayout>>(),
                push_constant_ranges: &[],
            },
        );

        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let descriptor = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: vertex_state,
            primitive: primitive_state,
            depth_stencil: depth_stencil_state,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(fragment_state),
            multiview: None,
        };

        pass_encoder
            .encoder
            .get_device()
            .create_render_pipeline(&descriptor)
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

pub struct FEditorRenderer {
    window: RcMut<winit::window::Window>,

    egui_renderer: egui_wgpu::Renderer,
    pass_buffer_view: FBufferView,

    frame_data: Option<(Vec<egui::ClippedPrimitive>, egui::FullOutput)>,
    user_textures: Vec<egui::TextureId>,

    scale_factor: f32,
}

pub struct FEguiGraphicsContext<'frame_encoder, 'command_encoder, 'device_encoder, 'editor_renderer>
{
    frame_encoder: &'frame_encoder mut FFrameEncoder<'command_encoder, 'device_encoder>,
    editor_renderer: &'editor_renderer mut FEditorRenderer,
}

impl FEguiGraphicsContext<'_, '_, '_, '_> {
    pub fn register_texture_online(&mut self, texture: &RcMut<FTexture>) -> egui::TextureId {
        let uitex = self
            .editor_renderer
            .register_texture_online(FTextureView::new(texture.clone()), self.frame_encoder);
        uitex
    }

    pub fn image_with_scale(
        &mut self,
        ui: &mut egui::Ui,
        texture: &RcMut<FTexture>,
        scale: f32,
    ) -> egui::Response {
        let uitex = self.register_texture_online(texture);
        let tex_ref = texture.borrow();
        ui.image(
            uitex,
            [
                tex_ref.get_width() as f32 * scale / self.editor_renderer.scale_factor,
                tex_ref.get_height() as f32 * scale / self.editor_renderer.scale_factor,
            ],
        )
    }

    pub fn image(&mut self, ui: &mut egui::Ui, texture: &RcMut<FTexture>) -> egui::Response {
        self.image_with_scale(ui, texture, 1.0)
    }
}

impl FEditorRenderer {
    pub fn new(
        window: &RcMut<winit::window::Window>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            window: window.clone(),
            user_textures: vec![],
            frame_data: None,
            egui_renderer: egui_wgpu::Renderer::new(device, surface_format, None, 1),
            pass_buffer_view: FBufferView::new_uniform(FBuffer::new_and_manage(
                BBufferUsages::Uniform,
            )),
            scale_factor: window.borrow().scale_factor() as f32,
        }
    }

    pub fn get_pass(&self) -> FPass {
        let bg_color = hoo_engine()
            .borrow()
            .egui_context
            .borrow()
            .style()
            .visuals
            .extreme_bg_color;

        let mut pass = FPass::new(self.pass_buffer_view.clone());
        pass.set_color_attachments(vec![FAttachment {
            texture_view: FTextureView::new_swapchain_view(),
            load_op: ELoadOp::Clear, // add a switch
            store_op: EStoreOp::Store,
            clear_value: FClearValue::Float4 {
                r: bg_color.r() as f32 / 255.0,
                g: bg_color.g() as f32 / 255.0,
                b: bg_color.b() as f32 / 255.0,
                a: 1.0,
            },
        }]);
        pass
    }

    pub fn register_texture_online(
        &mut self,
        texture: FTextureView,
        frame_encoder: &mut FFrameEncoder,
    ) -> egui::TextureId {
        let out = self.egui_renderer.register_native_texture(
            &frame_encoder.encoder.device,
            &texture.get_device_texture_view(&frame_encoder.encoder),
            wgpu::FilterMode::Linear,
        );
        self.user_textures.push(out);
        out
    }

    pub fn prepare(&mut self, frame_encoder: &mut FFrameEncoder) {
        let hoo_engine_rc = hoo_engine();
        let hoo_engine = hoo_engine_rc.borrow();

        let egui_context = hoo_engine.get_egui_context_mut();
        egui_context.begin_frame(hoo_engine.take_egui_input());
        hoo_engine.get_editor().draw(
            &egui_context,
            FEguiGraphicsContext {
                frame_encoder,
                editor_renderer: self,
            },
        );
        let full_output = egui_context.end_frame();
        let paint_jobs = egui_context.tessellate(full_output.shapes.clone());

        let scale_factor = self.window.borrow().scale_factor() as f32;
        let size = frame_encoder.encoder.get_swapchain_size();
        self.scale_factor = scale_factor;

        let descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [size.0, size.1],
            pixels_per_point: scale_factor,
        };

        for tex in full_output.textures_delta.set.iter() {
            self.egui_renderer.update_texture(
                &frame_encoder.encoder.device,
                &frame_encoder.encoder.queue,
                tex.0,
                &tex.1,
            );
        }

        self.egui_renderer.update_buffers(
            &frame_encoder.encoder.device,
            &frame_encoder.encoder.queue,
            &mut frame_encoder.command_encoder,
            &paint_jobs,
            &descriptor,
        );

        self.frame_data = Some((paint_jobs, full_output));
    }

    pub fn encode<'command_encoder, 'pass>(
        &'command_encoder mut self,
        mut pass_encoder: FPassEncoder<'pass, '_>,
    ) where
        'command_encoder: 'pass,
    {
        let scale_factor = self.window.borrow().scale_factor() as f32;
        let size = pass_encoder.encoder.get_swapchain_size();
        let descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [size.0, size.1],
            pixels_per_point: scale_factor,
        };

        let frame_data = self.frame_data.clone().unwrap();

        let paint_jobs = pass_encoder.transient_resources_cache.alloc(frame_data.0);
        self.egui_renderer
            .render(&mut pass_encoder.render_pass, paint_jobs, &descriptor);
    }

    pub fn free(&mut self) {
        let frame_data = self.frame_data.take().unwrap();

        for tex in frame_data.1.textures_delta.free {
            self.egui_renderer.free_texture(&tex);
        }

        for tex in self.user_textures.iter() {
            self.egui_renderer.free_texture(tex);
        }

        self.user_textures.clear();
    }
}

pub struct FDeviceEncoder {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,

    surface: wgpu::Surface,
    surface_present_mode: wgpu::PresentMode,
    surface_alpha_mode: wgpu::CompositeAlphaMode,
    swapchain_texture: RefCell<Option<wgpu::SurfaceTexture>>,
    swapchain_size: (u32, u32),
    swapchain_format: ETextureFormat,

    bind_group_layouts: Vec<wgpu::BindGroupLayout>,

    uniform_buffer_view_global: Option<FBufferView>,
    uniform_buffer_view_task: Option<FBufferView>,
    default_uniform_buffer_view: Option<FBufferView>,

    editor: Option<RcMut<FEditorRenderer>>,
    window: RcMut<winit::window::Window>,
}

pub struct FFrameEncoder<'command_encoder, 'device_encoder> {
    // access using a function
    encoder: &'device_encoder mut FDeviceEncoder,
    // private
    command_encoder: wgpu::CommandEncoder,
    resources: &'command_encoder Vec<Ref<'command_encoder, dyn TGPUResource>>,
}
// paint_jobs
pub struct FPassEncoder<'pass, 'device_encoder> {
    // access using a function
    encoder: &'device_encoder FDeviceEncoder,

    // keep them private
    pass: FPass,
    render_pass: wgpu::RenderPass<'pass>,
    // 这些资源实际上的生命周期比 'pass 长
    resources: &'pass Vec<Ref<'pass, dyn TGPUResource>>,

    transient_resources_cache: &'pass bumpalo::Bump,
    // render_pipeline_cache: &'command_encoder typed_arena::Arena<wgpu::RenderPipeline>,
    // bind_group_cache: &'command_encoder typed_arena::Arena<wgpu::BindGroup>,
    // egui_paint_jobs_cache: &'command_encoder typed_arena::Arena<Vec<egui::ClippedPrimitive>>  // 或许用 untyped 更好? 避免这边放一堆类型
}

impl FDeviceEncoder {
    // getter
    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn get_editor_renderer_ref(&self) -> Ref<FEditorRenderer> {
        self.editor.as_ref().unwrap().borrow()
    }

    pub fn get_editor_renderer_mut(&self) -> RefMut<FEditorRenderer> {
        self.editor.as_ref().unwrap().borrow_mut()
    }

    pub fn get_swapchain_texture(&self) -> Ref<Option<wgpu::SurfaceTexture>> {
        {
            let mut swapchain_texture = self.swapchain_texture.borrow_mut();
            if swapchain_texture.is_none() {
                *swapchain_texture = Some(self.surface.get_current_texture().unwrap());
            }
        }
        return self.swapchain_texture.borrow();
    }

    pub fn get_swapchain_format(&self) -> ETextureFormat {
        self.swapchain_format
    }

    fn get_bind_group_layouts(&self) -> &Vec<wgpu::BindGroupLayout> {
        &self.bind_group_layouts
    }

    pub fn resize_surface(&mut self) {
        let window = self.window.borrow();
        let new_size = (window.inner_size().width, window.inner_size().height);
        if new_size == self.swapchain_size {
            return;
        }
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.swapchain_format.into(),
            width: new_size.0,
            height: new_size.1,
            present_mode: self.surface_present_mode,
            alpha_mode: self.surface_alpha_mode,
            view_formats: vec![],
        };
        self.surface =
            unsafe { self.instance.create_surface(self.window.borrow().deref()) }.unwrap();
        self.surface.configure(&self.device, &config);
        self.swapchain_size = new_size;
    }

    // impl

    pub fn new(window: &RcMut<winit::window::Window>) -> Self {
        futures::executor::block_on(Self::new_async(window))
    }

    pub async fn new_async(window: &RcMut<winit::window::Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(window.borrow().deref()) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let size = window.borrow().inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let surface_texture = surface.get_current_texture().unwrap();

        let bind_group_layouts = Self::make_bind_group_layouts(&device);

        Self {
            instance,
            device,
            queue,

            surface,
            surface_present_mode: surface_caps.present_modes[0],
            surface_alpha_mode: surface_caps.alpha_modes[0],

            swapchain_size: (size.width, size.height),
            swapchain_texture: RefCell::new(Some(surface_texture)),
            swapchain_format: ETextureFormat::try_from(surface_format).unwrap(),

            bind_group_layouts,
            uniform_buffer_view_global: None,
            uniform_buffer_view_task: None,
            default_uniform_buffer_view: None,

            editor: None,
            window: window.clone(),
        }
    }

    pub fn prepare(&mut self) {
        let editor = FEditorRenderer::new(&self.window, &self.device, self.swapchain_format.into());
        self.editor = Some(rcmut!(editor));

        self.default_uniform_buffer_view = Some(FBufferView::new_uniform(FBuffer::new_and_manage(
            BBufferUsages::Uniform,
        )));

        self.set_global_uniform_buffer_view(
            self.default_uniform_buffer_view.as_ref().unwrap().clone(),
        );
    }

    pub fn get_swapchain_size(&self) -> (u32, u32) {
        self.swapchain_size
    }

    // 所有 pipeline 的 bind group layout 都是一致的, 若干个 cbuffer + 若干个贴图
    // cbuffer: MaterialUniform, DrawCallUniform, PassUniform, TaskUniform, GlobalUniform
    // 贴图：固定数量

    // DrawCallUniform: 在 DrawCommand 中提供，大小可变
    // PassUniform: 在 Pass 中提供，大小可变
    // TaskUniform: 暂时没想好如何做抽象，先通过 SetTaskUniform 设置。类似于 Viewport 的概念，表示一个完整的渲染管线。
    // GlobalUniform: 通过 SetGlobalUniformBuffer 设置，大小固定
    fn make_bind_group_layouts(device: &wgpu::Device) -> Vec<wgpu::BindGroupLayout> {
        let mut bind_group_layout_entries = vec![];
        for i in 0..EUniformBufferType::COUNT {
            let entry = wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            bind_group_layout_entries.push(entry);
        }

        let desc = wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroup-0"),
            entries: &bind_group_layout_entries,
        };

        let out = device.create_bind_group_layout(&desc);

        vec![out]
    }

    pub fn set_global_uniform_buffer_view(&mut self, buffer: FBufferView) {
        self.uniform_buffer_view_global = Some(buffer);
    }

    pub fn encode_frame<F: FnOnce(&mut FFrameEncoder)>(&mut self, frame_closure: F) {
        let command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Command Encoder"),
            });

        let res = hoo_engine()
            .borrow()
            .get_resources()
            .prepare_gpu_resources(self);

        let res_ref = res.iter().map(|x| x.borrow()).collect::<Vec<_>>();

        let editor = self.editor.as_ref().unwrap().clone();

        let command_encoder = {
            let mut frame_encoder = FFrameEncoder {
                encoder: self,
                command_encoder,
                resources: &res_ref,
            };
            frame_closure(&mut frame_encoder);

            let mut editor = editor.borrow_mut();
            let pass = editor.get_pass();

            editor.prepare(&mut frame_encoder);
            frame_encoder.encode_render_pass(pass, |pass_encoder| {
                editor.encode(pass_encoder);
            });
            editor.free();
            frame_encoder.command_encoder
        };

        // let swapchain_view = FTextureView::new_swapchain_view().get_device_texture_view(&self);

        self.queue.submit(std::iter::once(command_encoder.finish()));
        self.present();

        self.resize_surface();
    }

    fn present(&mut self) {
        let _ = self.get_swapchain_texture();
        self.swapchain_texture.take().unwrap().present();
    }
}

impl<'command_encoder, 'device_encoder> FFrameEncoder<'command_encoder, 'device_encoder> {
    pub fn set_task_uniform_buffer_view(&mut self, buffer: FBufferView) {
        self.encoder.uniform_buffer_view_task = Some(buffer);
    }

    pub fn encode_render_pass<F: FnOnce(FPassEncoder)>(
        &mut self,
        render_pass: FPass,
        pass_closure: F,
    ) {
        self.encode_render_pass_with_argument(render_pass, |a, _| pass_closure(a), ());
    }

    pub fn encode_render_pass_with_argument<E, F: FnOnce(FPassEncoder, E)>(
        &mut self,
        render_pass: FPass,
        pass_closure: F,
        extra_data: E,
    ) {
        let color_attachments_views: Vec<_> = render_pass
            .get_color_attachments()
            .iter()
            .map(|x| x.texture_view.get_device_texture_view(self.encoder))
            .collect();

        let color_attachments: Vec<_> = render_pass
            .get_color_attachments()
            .iter()
            .zip(color_attachments_views.iter())
            .map(|(x, y)| {
                let z = wgpu::RenderPassColorAttachment {
                    view: y,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: x.load_op.to_wgpu_color(&x.clear_value),
                        store: x.store_op.store(),
                    },
                };
                Some(z)
            })
            .collect();

        let depth_attachment_view = render_pass
            .get_depth_stencil_attachment()
            .as_ref()
            .map(|x| x.texture_view.get_device_texture_view(self.encoder));

        let depth_attachment = render_pass
            .get_depth_stencil_attachment()
            .iter()
            .zip(depth_attachment_view.iter())
            .map(|(dsv, view)| wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(wgpu::Operations {
                    load: dsv.load_op.to_wgpu_value(dsv.clear_value.clone()),
                    store: dsv.store_op.store(),
                }),
                stencil_ops: None,
            })
            .collect::<Vec<wgpu::RenderPassDepthStencilAttachment>>()
            .pop();

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: depth_attachment,
        };

        let pass = self
            .command_encoder
            .begin_render_pass(&render_pass_descriptor);

        let transient_resources_cache = bumpalo::Bump::new();

        let pass_encoder = FPassEncoder {
            pass: render_pass,
            resources: self.resources,
            encoder: self.encoder,
            render_pass: pass,

            transient_resources_cache: &transient_resources_cache,
        };

        pass_closure(pass_encoder, extra_data);
    }

    pub fn get_device_encoder(&mut self) -> &mut FDeviceEncoder {
        self.encoder
    }
}

impl<'command_encoder, 'device_encoder> FPassEncoder<'command_encoder, 'device_encoder> {
    pub fn setup_pipeline(
        &mut self,
        vertex_entries: &[FVertexEntry],
        shader_module: &RcMut<FShaderModule>,
    ) {
        let mut pipeline = FPipeline::new();
        let device_pipeline =
            pipeline.create_device_resource_with_pass(self, vertex_entries, shader_module);
        let pipeline_ref = self.transient_resources_cache.alloc(device_pipeline);
        self.render_pass.set_pipeline(pipeline_ref);
    }

    pub fn create_bind_group_entry_of_buffer(
        &self,
        binding: u32,
        view: &FBufferView,
    ) -> wgpu::BindGroupEntry {
        let buffer_id = view.get_buffer().borrow().get_consolidation_id();
        let buffer_ref = self.resources[buffer_id as usize].as_buffer().unwrap();

        let entry = wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: buffer_ref.get_device_buffer(),
                offset: view.get_offset() as wgpu::BufferAddress,
                size: None,
            }),
        };

        entry
    }

    pub fn draw(&mut self, draw_command: &FDrawCommand) {
        let vertex_buffers = draw_command.get_vertex_buffers();
        let index_buffer_view = draw_command.get_index_buffer_view();

        for vertex_buffer_entry in vertex_buffers.iter() {
            let vertex_buffer_view = &vertex_buffer_entry.view;
            let vertex_buffer = vertex_buffer_view.get_buffer();
            let vertex_buffer_ref = self.resources
                [vertex_buffer.borrow().get_consolidation_id() as usize]
                .as_buffer()
                .unwrap();
            let vertex_buffer_offset = vertex_buffer_view.get_offset();
            let vertex_buffer_size = vertex_buffer_view.size();
            self.render_pass.set_vertex_buffer(
                vertex_buffer_entry.location,
                vertex_buffer_ref
                    .get_device_buffer()
                    .slice(vertex_buffer_offset..(vertex_buffer_offset + vertex_buffer_size)),
            );
        }

        let index_buffer = index_buffer_view.get_buffer();
        let index_buffer_ref = self.resources
            [index_buffer.borrow().get_consolidation_id() as usize]
            .as_buffer()
            .unwrap();
        let index_buffer_offset = index_buffer_view.get_offset();
        let index_buffer_size = index_buffer_view.size();

        self.render_pass.set_index_buffer(
            index_buffer_ref
                .get_device_buffer()
                .slice(index_buffer_offset..(index_buffer_offset + index_buffer_size)),
            wgpu::IndexFormat::Uint32,
        );

        let bind_group_entries = [
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
                self.pass.get_uniform_buffer_view(),
            ),
            self.create_bind_group_entry_of_buffer(
                EUniformBufferType::Task as u32,
                self.encoder.uniform_buffer_view_task.as_ref().unwrap(),
            ),
            self.create_bind_group_entry_of_buffer(
                EUniformBufferType::Global as u32,
                self.encoder.uniform_buffer_view_global.as_ref().unwrap(),
            ),
        ];

        let bindgroup_0 = self
            .encoder
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.encoder.bind_group_layouts[0],
                entries: &bind_group_entries,
            });

        let bindgroup_0_ref = self.transient_resources_cache.alloc(bindgroup_0);

        self.render_pass.set_bind_group(0, bindgroup_0_ref, &[]);

        self.render_pass
            .draw_indexed(0..draw_command.get_index_count() as u32, 0, 0..1);
    }
}
