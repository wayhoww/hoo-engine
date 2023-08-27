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

    rt_color: FTextureView,

    uniform_view: FBufferView,

    task_uniform_buffer: RcMut<FBuffer>,
    task_uniform_view: FBufferView,
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

        Self {
            pass1,
            pass2,
            cursor_pass: FCursorPass::new(),
            compute_shader: cs,
            uniform_view,
            task_uniform_buffer,
            task_uniform_view,
            rt_color: FTextureView::new_swapchain_view(),
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

        let size = match &context.render_target {
            crate::object::objects::HCameraTarget::Screen => {
                let swapchain_image_view = FTextureView::new_swapchain_view();
                self.rt_color = swapchain_image_view.clone();

                self.pass1.set_color_attachments(vec![FAttachment::new(
                    swapchain_image_view.clone(),
                    ELoadOp::Clear,
                    EStoreOp::Store,
                )
                .set_clear_value(FClearValue::Float4 {
                    r: 0.1,
                    g: 0.0,
                    b: 0.1,
                    a: 1.0,
                })
                .clone()]);

                // self.pass2.set_color_attachments(vec![FAttachment::new(
                //     swapchain_image_view.clone(),
                //     ELoadOp::Load,
                //     EStoreOp::Store,
                // )]);

                // context.
            }
            crate::object::objects::HCameraTarget::Texture(tex) => {
                // let color_texture = FTexture::new_and_manage(
                //     ETextureFormat::Bgra8Unorm,
                //     BTextureUsages::Attachment | BTextureUsages::Sampled,
                // );
                let color_texture_view = FTextureView::new(tex.clone());
                self.rt_color = color_texture_view.clone();

                self.pass1.set_color_attachments(vec![FAttachment::new(
                    color_texture_view.clone(),
                    ELoadOp::Clear,
                    EStoreOp::Store,
                )
                .set_clear_value(FClearValue::Float4 {
                    r: 0.1,
                    g: 0.0,
                    b: 0.1,
                    a: 1.0,
                })
                .clone()]);

                // self.pass2.set_color_attachments(vec![FAttachment::new(
                //     color_texture_view.clone(),
                //     ELoadOp::Load,
                //     EStoreOp::Store,
                // )]);
            }
        };

        let depth_stencil_texture = FTexture::new_and_manage(
            ETextureFormat::Depth24PlusStencil8,
            BTextureUsages::Attachment,
        );

        depth_stencil_texture
            .borrow_mut()
            .set_size(context.render_target_size);
        let depth_stencil_texture_view = FTextureView::new(depth_stencil_texture.clone());
        self.pass1.set_depth_stencil_attachment(FAttachment {
            texture_view: depth_stencil_texture_view.clone(),
            load_op: ELoadOp::Clear,
            store_op: EStoreOp::Discard,
            clear_value: FClearValue::Float(1f32),
        });
        // self.pass2.set_depth_stencil_attachment(FAttachment {
        //     texture_view: depth_stencil_texture_view.clone(),
        //     load_op: ELoadOp::Clear,
        //     store_op: EStoreOp::Discard,
        //     clear_value: FClearValue::Float(1f32),
        // });
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
            pass_encoder.dispatch(&self.compute_shader, (3, 1, 2));
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
