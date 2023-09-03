use std::rc::Rc;

use crate::device::graphics::*;

use crate::device::io::load_string;
use crate::object::objects::FShaderLight;
use crate::{hoo_engine, utils::*};

use nalgebra_glm as glm;

use super::affiliate::FCursorPass;
use super::FPipelineContextData;

#[derive(Default, Clone)]
pub struct FGraphicsPipelineProperties {
    pub hovered_object_id: RcMut<Option<u32>>,
}

pub struct FGraphicsPipeline {
    forward_opaque_pass: FGraphicsPass,
    model_axis_pass: FGraphicsPass,

    pass2: FComputePass,
    cursor_pass: FCursorPass,
    compute_shader: RcMut<FShaderModule>,

    uav: FTextureView,

    rt_color: FTextureView,
    rt_depth: FTextureView,
    rt_object_id: FTextureView,

    uniform_view: FBufferView,

    task_uniform_buffer: RcMut<FBuffer>,
    task_uniform_view: FBufferView,

    cursor_uniform_buffer: RcMut<FBuffer>,
    cursor_uniform_view: FBufferView,

    readback_src_buffer: RcMut<FBuffer>,
    readback_src_buffer_view: FBufferView,

    readback_buffer: RcMut<FBuffer>,

    properties: FGraphicsPipelineProperties,
}

#[derive(Clone, Debug)]
#[repr(C, packed)]
pub struct FTaskUnifromBuffer {
    light_count: u32,
    _padding_0: [u32; 3],
    lights: [FShaderLight; 16],
}

impl FGraphicsPipeline {
    pub fn new<'a>() -> Self {
        // faked
        let default_uniform_buffer = FBuffer::new_and_manage(BBufferUsages::Uniform);
        default_uniform_buffer.borrow_mut().resize(1024);
        let uniform_view = FBufferView::new_uniform(default_uniform_buffer.clone());

        // task
        let task_uniform_buffer = FBuffer::new_and_manage(BBufferUsages::Uniform);
        task_uniform_buffer
            .borrow_mut()
            .resize(std::mem::size_of::<FShaderLight>() as u64);
        let task_uniform_view = FBufferView::new_uniform(task_uniform_buffer.clone());

        let forward_opaque_pass = FGraphicsPass::new(uniform_view.clone());
        let model_axis_pass = FGraphicsPass::new(uniform_view.clone());
        let pass2 = FComputePass::default();

        let cs = FShaderModule::new_and_manage(load_string("shaders/compute.wgsl").unwrap());
        cs.borrow_mut().set_compute_stage_entry("main".into());

        let uav_tex = FTexture::new_and_manage(
            ETextureFormat::Rgba32Float,
            BTextureUsages::UnorderedAccess | BTextureUsages::Sampled,
        );

        uav_tex.borrow_mut().set_size((512, 512));

        let uav_tex_view = FTextureView::new(uav_tex.clone());

        let readback_buffer = FBuffer::new_and_manage(BBufferUsages::MapRead);
        readback_buffer.borrow_mut().resize(4);

        let readback_src_buffer =
            FBuffer::new_and_manage(BBufferUsages::Storage | BBufferUsages::CopySrc);
        readback_src_buffer.borrow_mut().resize(4);

        let readback_src_bufferview =
            FBufferView::new_with_type(readback_src_buffer.clone(), EBufferViewType::Storage);

        let cursor_uniform_buffer = FBuffer::new_and_manage(BBufferUsages::Uniform);
        cursor_uniform_buffer
            .borrow_mut()
            .update_by_array(&[hoo_engine()
                .borrow()
                .get_editor()
                .get_state()
                .main_viewport_cursor_position
                .unwrap_or((0.0f32, 0.0f32))]);
        let cursor_uniform_view = FBufferView::new_uniform(cursor_uniform_buffer.clone());

        let texture_object_id = FTexture::new_and_manage(
            ETextureFormat::R32Uint,
            BTextureUsages::Attachment | BTextureUsages::Sampled,
        );

        texture_object_id.borrow_mut().set_size((512, 512));

        let texture_depth = FTexture::new_and_manage(
            ETextureFormat::Depth24PlusStencil8,
            BTextureUsages::Attachment,
        );

        texture_depth.borrow_mut().set_size((512, 512));

        Self {
            forward_opaque_pass,
            model_axis_pass,
            pass2,
            cursor_pass: FCursorPass::new(),
            compute_shader: cs,
            uniform_view,
            task_uniform_buffer,
            task_uniform_view,
            rt_color: FTextureView::new_swapchain_view(),
            rt_depth: FTextureView::new(texture_depth),
            rt_object_id: FTextureView::new(texture_object_id),
            uav: uav_tex_view,
            readback_buffer: readback_buffer,
            readback_src_buffer: readback_src_buffer,
            readback_src_buffer_view: readback_src_bufferview,
            cursor_uniform_buffer,
            cursor_uniform_view,
            properties: FGraphicsPipelineProperties::default(),
        }
    }

    pub fn prepare(&mut self, context: &mut FPipelineContextData) {
        let mat_view = {
            let camera_rotation: glm::Mat4 =
                glm::make_mat3(&[-1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]).to_homogeneous();
            let view_inv = context.camera_transform * camera_rotation;
            view_inv.try_inverse().unwrap()
        };

        let mat_proj = context.camera_projection;

        let mut task_uniform_buffer_data = FTaskUnifromBuffer {
            light_count: (context.lights.len() as u32).into(),
            _padding_0: [0; 3],
            lights: [FShaderLight::default(); 16],
        };

        for (i, light) in context.lights.iter().enumerate() {
            task_uniform_buffer_data.lights[i] = light.clone();
        }

        self.task_uniform_buffer
            .borrow_mut()
            .update_by_struct(&task_uniform_buffer_data);

        for render_object in context.render_objects.iter_mut() {
            // 这个混在一起会不会有些奇怪。以后这类问题可以看看其他引擎的做法
            render_object
                .set_transform_view(mat_view)
                .set_transform_projection(mat_proj)
                .update_uniform_buffer();
        }

        let viewport_size = match &context.render_target {
            crate::object::objects::HCameraTarget::Screen => {
                let swapchain_image_view = FTextureView::new_swapchain_view();
                self.rt_color = swapchain_image_view.clone();
                context.render_target_size
            }
            crate::object::objects::HCameraTarget::Texture(tex) => {
                let color_texture_view = FTextureView::new(tex.clone());
                self.rt_color = color_texture_view.clone();
                tex.borrow().size()
            }
        };

        self.cursor_uniform_buffer
            .borrow_mut()
            .update_by_array(&[hoo_engine()
                .borrow()
                .get_editor()
                .get_state()
                .main_viewport_cursor_position
                .map(|(x, y)| {
                    (
                        (x.round() as i32).clamp(0, viewport_size.0 as i32 - 1) as u32,
                        (y.round() as i32).clamp(0, viewport_size.1 as i32 - 1) as u32,
                    )
                })
                .unwrap_or((0, 0))]);

        self.rt_depth
            .get_texture()
            .borrow_mut()
            .set_size(viewport_size);
        self.rt_object_id
            .get_texture()
            .borrow_mut()
            .set_size(viewport_size);

        let mut object_id_attachment = FAttachment::new_write_to_view(self.rt_object_id.clone());
        object_id_attachment.set_clear_value(FClearValue::Float(u32::MAX as f64));

        self.forward_opaque_pass.set_color_attachments(vec![
            FAttachment::new_write_to_view(self.rt_color.clone()),
            object_id_attachment,
        ]);

        self.forward_opaque_pass.set_depth_stencil_attachment(FAttachment {
            texture_view: self.rt_depth.clone(),
            load_op: ELoadOp::Clear,
            store_op: EStoreOp::Discard,
            clear_value: FClearValue::Float(1f64),
        });

        self.model_axis_pass.set_color_attachments(vec![FAttachment::new_append_to_view(self.rt_color.clone())]);
    }

    pub fn draw(&mut self, frame_encoder: &mut FFrameEncoder, context: &mut FPipelineContextData) {
        frame_encoder.set_task_uniform_buffer_view(self.task_uniform_view.clone());

        frame_encoder.encode_render_pass(self.forward_opaque_pass.clone(), |mut pass_encoder| {
            for render_object in context.render_objects.iter() {
                if render_object.get_flags().contains(BRenderObjectFlags::NORMAL_OPAQUE) {
                    render_object.encode(&mut pass_encoder, "base");
                }
            }
        });

        frame_encoder.encode_render_pass(self.model_axis_pass.clone(), |mut pass_encoder| {
            for render_object in context.render_objects.iter() {
                if render_object.get_flags().contains(BRenderObjectFlags::MODEL_AXIS) {
                    render_object.encode(&mut pass_encoder, "model_axis");
                }
            }
        });

        if hoo_engine()
            .borrow()
            .get_editor()
            .get_state()
            .main_viewport_cursor_position
            .is_some()
        {
            frame_encoder.encode_compute_pass(self.pass2.clone(), |mut pass_encoder| {
                pass_encoder
                    .get_bind_group_descriptor()
                    .add_buffer(0, self.readback_src_buffer_view.clone())
                    .add_sampled_texture(1, self.rt_object_id.clone());
                pass_encoder.dispatch(&self.compute_shader, (1, 1, 1), &self.cursor_uniform_view);
            });

            frame_encoder.copy_buffer(&self.readback_src_buffer, &self.readback_buffer);

            let hovered_object_id_weak = Rc::downgrade(&self.properties.hovered_object_id);
            frame_encoder.read_back(&self.readback_buffer, move |data: Option<Vec<u32>>| {
                hovered_object_id_weak.upgrade().map(|hovered_object_id| {
                    if let Some(data) = data {
                        let object_id = data[0];
                        if object_id != u32::MAX {
                            *hovered_object_id.borrow_mut() = Some(object_id);
                        } else {
                            *hovered_object_id.borrow_mut() = None;
                        }
                    } else {
                        *hovered_object_id.borrow_mut() = None;
                    }
                });
            });
        } else {
            *self.properties.hovered_object_id.borrow_mut() = None;
        }
    }

    pub fn get_properties(&self) -> &FGraphicsPipelineProperties {
        &self.properties
    }
}
