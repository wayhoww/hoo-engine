use std::cell::{Ref, RefCell};
use std::convert::TryFrom;

use crate::utils::*;
use crate::*;

use super::resource::*;

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
            .map(|(entry, attr)| {
                let layout = wgpu::VertexBufferLayout {
                    array_stride: entry.stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: attr,
                };
                layout
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

pub struct FDeviceEncoder {
    device: wgpu::Device,
    queue: wgpu::Queue,

    surface: wgpu::Surface,
    swapchain_texture: RefCell<Option<wgpu::SurfaceTexture>>,
    swapchain_size: (u32, u32),
    swapchain_format: ETextureFormat,

    bind_group_layouts: Vec<wgpu::BindGroupLayout>,

    uniform_buffer_view_global: Option<FBufferView>,
    uniform_buffer_view_task: Option<FBufferView>,
}

pub struct FPassEncoder<'a: 'c, 'b: 'c, 'c, 'd: 'c, 'e> {
    // access using a function
    encoder: &'a FDeviceEncoder,

    // keep them private
    pass: &'b FPass,
    render_pass: wgpu::RenderPass<'c>,
    resources: &'d Vec<Ref<'e, dyn TGPUResource>>,

    render_pipeline_cache: &'d typed_arena::Arena<wgpu::RenderPipeline>,
    bind_group_cache: &'d typed_arena::Arena<wgpu::BindGroup>,
}

pub struct FFrameEncoder<'a, 'b, 'c> {
    // access using a function
    encoder: &'a mut FDeviceEncoder,
    // private
    command_encoder: wgpu::CommandEncoder,
    resources: &'b Vec<Ref<'c, dyn TGPUResource>>,
}

impl FDeviceEncoder {
    // getter
    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
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

    // impl

    pub fn new(window: &winit::window::Window) -> Self {
        futures::executor::block_on(Self::new_async(window))
    }

    pub async fn new_async(window: &winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

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

        let size = window.inner_size();
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
            device,
            queue,
            surface,
            swapchain_size: (size.width, size.height),
            swapchain_texture: RefCell::new(Some(surface_texture)),
            swapchain_format: ETextureFormat::try_from(surface_format).unwrap(),
            bind_group_layouts,
            uniform_buffer_view_global: None,
            uniform_buffer_view_task: None,
        }
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

    pub fn set_task_uniform_buffer_view(&mut self, buffer: FBufferView) {
        self.uniform_buffer_view_task = Some(buffer);
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

        let command_encoder = {
            let mut frame_encoder = FFrameEncoder {
                encoder: self,
                command_encoder,
                resources: &res_ref,
            };

            frame_closure(&mut frame_encoder);

            frame_encoder.command_encoder
        };

        self.queue.submit(std::iter::once(command_encoder.finish()));
    }

    pub fn present(&mut self) {
        let _ = self.get_swapchain_texture();
        self.swapchain_texture.take().unwrap().present();
    }
}

impl FFrameEncoder<'_, '_, '_> {
    pub fn encode_render_pass<F: FnOnce(&mut FPassEncoder)>(
        &mut self,
        render_pass: &FPass,
        pass_closure: F,
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

        let render_pipeline_cache = typed_arena::Arena::new();
        let bindgroup_cache = typed_arena::Arena::new();

        let mut pass_encoder = FPassEncoder {
            pass: render_pass,
            resources: self.resources,
            encoder: self.encoder,
            render_pass: pass,

            render_pipeline_cache: &render_pipeline_cache,
            bind_group_cache: &bindgroup_cache,
        };

        pass_closure(&mut pass_encoder);
    }

    pub fn get_device_encoder(&mut self) -> &mut FDeviceEncoder {
        self.encoder
    }
}

impl<'c> FPassEncoder<'_, '_, 'c, '_, '_> {
    pub fn setup_pipeline(
        &mut self,
        vertex_entries: &[FVertexEntry],
        shader_module: &RcMut<FShaderModule>,
    ) {
        let mut pipeline = FPipeline::new();
        let device_pipeline =
            pipeline.create_device_resource_with_pass(self, vertex_entries, shader_module);
        let pipeline_ref = self.render_pipeline_cache.alloc(device_pipeline);
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

        let bindgroup_0_ref = self.bind_group_cache.alloc(bindgroup_0);

        self.render_pass.set_bind_group(0, bindgroup_0_ref, &[]);

        self.render_pass
            .draw_indexed(0..draw_command.get_index_count() as u32, 0, 0..1);
    }
}
