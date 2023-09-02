use crate::device::graphics::*;

use crate::device::io::load_string;
use crate::object::objects::FShaderLight;
use crate::{hoo_engine, utils::*};

use nalgebra_glm as glm;

use super::affiliate::FCursorPass;
use super::FPipelineContext;

pub struct FGraphicsPipeline {
    pass1: FGraphicsPass,
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

        // // pass
        // let size = (512, 512);

        // let color_texture = FTexture::new_and_manage(
        //     ETextureFormat::Bgra8Unorm,
        //     BTextureUsages::Attachment | BTextureUsages::Sampled,
        // );
        // color_texture.borrow_mut().set_size(size);
        // hoo_engine().borrow().get_editor_mut().main_viewport_texture = Some(color_texture.clone());
        // let color_texture_view = FTextureView::new(color_texture.clone());

        // let depth_texture = FTexture::new_and_manage(
        //     ETextureFormat::Depth24PlusStencil8,
        //     BTextureUsages::Attachment,
        // );
        // depth_texture.borrow_mut().set_size(size);
        // let depth_texture_view = FTextureView::new(depth_texture.clone());

        let pass1 = FGraphicsPass::new(uniform_view.clone());
        let pass2 = FComputePass::default();
        // pass1.set_depth_stencil_attachment(FAttachment {
        //     texture_view: depth_texture_view.clone(),
        //     load_op: ELoadOp::Clear,
        //     store_op: EStoreOp::Discard,
        //     clear_value: FClearValue::Float(1f32),
        // });
        // let mut color_attachment =
        //     FAttachment::new(color_texture_view.clone(), ELoadOp::Clear, EStoreOp::Store);
        // color_attachment.set_clear_value(FClearValue::Float4 {
        //     r: 0.1,
        //     g: 0.0,
        //     b: 0.1,
        //     a: 1.0,
        // });
        // pass1.set_color_attachments(vec![color_attachment]);

        // pass2.set_depth_stencil_attachment(FAttachment {
        //     texture_view: depth_texture_view.clone(),
        //     load_op: ELoadOp::Clear,
        //     store_op: EStoreOp::Discard,
        //     clear_value: FClearValue::Float(1f32),
        // });
        // pass2.set_color_attachments(vec![FAttachment::new(
        //     color_texture_view.clone(),
        //     ELoadOp::Load,
        //     EStoreOp::Store,
        // )]);

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
            pass1,
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
        }
    }

    pub fn prepare(&mut self, context: &mut FPipelineContext) {
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

        self.pass1.set_color_attachments(vec![
            FAttachment::new_write_to_view(self.rt_color.clone()),
            FAttachment::new_write_to_view(self.rt_object_id.clone()),
        ]);

        self.pass1.set_depth_stencil_attachment(FAttachment {
            texture_view: self.rt_depth.clone(),
            load_op: ELoadOp::Clear,
            store_op: EStoreOp::Discard,
            clear_value: FClearValue::Float(1f32),
        });
    }

    pub fn draw(&mut self, frame_encoder: &mut FFrameEncoder, context: &mut FPipelineContext) {
        // encoder.encode_frame(|frame_encoder| {
        // let swapchain_image_view = FTextureView::new_swapchain_view();

        // self.pass1.set_color_attachments(vec![FAttachment::new(
        //     swapchain_image_view.clone(),
        //     ELoadOp::Clear,
        //     EStoreOp::Store,
        // )
        // .set_clear_value(FClearValue::Float4 {
        //     r: 0.1,
        //     g: 0.0,
        //     b: 0.1,
        //     a: 1.0,
        // })
        // .clone()]);

        // self.pass2.set_color_attachments(vec![FAttachment::new(
        //     swapchain_image_view.clone(),
        //     ELoadOp::Load,
        //     EStoreOp::Store,
        // )]);

        frame_encoder.set_task_uniform_buffer_view(self.task_uniform_view.clone());

        frame_encoder.encode_render_pass(self.pass1.clone(), |mut pass_encoder| {
            // self.render_object1.encode(pass_encoder, "base");

            for render_object in context.render_objects.iter() {
                render_object.encode(&mut pass_encoder, "base");
            }
        });

        frame_encoder.encode_compute_pass(self.pass2.clone(), |mut pass_encoder| {
            pass_encoder
                .get_bind_group_descriptor()
                .add_buffer(0, self.readback_src_buffer_view.clone())
                .add_sampled_texture(1, self.rt_object_id.clone());
            // .add_unordered_access(3, self.uav.clone());
            pass_encoder.dispatch(&self.compute_shader, (3, 1, 2), &self.cursor_uniform_view);
        });

        frame_encoder.copy_buffer(&self.readback_src_buffer, &self.readback_buffer);

        frame_encoder.read_back(&self.readback_buffer, |data: Option<Vec<u32>>| {
            println!("readback: {:?}", data);
        });

        // frame_encoder.encode_render_pass(self.cursor_pass.get_pass(self.rt_color.clone()), |mut pass_encoder| {
        //     self.cursor_pass.encode_pass(&mut pass_encoder);
        // });

        // frame_encoder.encode_render_pass(&self.pass2, |pass_encoder| {
        //     self.render_object2.encode(pass_encoder, "base");
        // });
        // });
    }
}
