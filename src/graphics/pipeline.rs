use crate::device::graphics::*;

use crate::object::objects::FShaderLight;
use crate::utils::*;

use nalgebra_glm as glm;

use super::FPipelineContext;

pub struct FGraphicsPipeline {
    pass1: FPass,
    pass2: FPass,

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
    pub fn new<'a>(encoder: &FDeviceEncoder) -> Self {
        // faked
        let default_uniform_buffer = FBuffer::new_and_manage(BBufferUsages::Uniform);
        default_uniform_buffer.borrow_mut().resize(1024);
        let uniform_view = FBufferView::new_uniform(default_uniform_buffer.clone());

        // task
        let task_uniform_buffer = FBuffer::new_and_manage(BBufferUsages::Uniform);
        task_uniform_buffer.borrow_mut().resize(std::mem::size_of::<FShaderLight>() as u64);
        let task_uniform_view = FBufferView::new_uniform(task_uniform_buffer.clone());

        // pass
        let depth_texture = FTexture::new_and_manage(
            ETextureFormat::Depth24PlusStencil8,
            BTextureUsages::Attachment,
        );
        depth_texture
            .borrow_mut()
            .set_size(encoder.get_swapchain_size());
        let depth_texture_view = FTextureView::new(depth_texture.clone());

        let mut pass1 = FPass::new(uniform_view.clone());
        let mut pass2 = FPass::new(uniform_view.clone());
        pass1.set_depth_stencil_attachment(FAttachment {
            texture_view: depth_texture_view.clone(),
            load_op: ELoadOp::Clear,
            store_op: EStoreOp::Discard,
            clear_value: FClearValue::Float(1f32),
        });
        pass2.set_depth_stencil_attachment(FAttachment {
            texture_view: depth_texture_view.clone(),
            load_op: ELoadOp::Clear,
            store_op: EStoreOp::Discard,
            clear_value: FClearValue::Float(1f32),
        });

        Self {
            pass1,
            pass2,
            uniform_view,
            task_uniform_buffer,
            task_uniform_view,
        }
    }

    pub fn draw(&mut self, encoder: &mut FDeviceEncoder, context: &mut FPipelineContext) {
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
            lights: [ FShaderLight::default(); 16 ],
        };

        for (i, light) in context.lights.iter().enumerate() {
            task_uniform_buffer_data.lights[i] = light.clone();
        }

        self.task_uniform_buffer.borrow_mut().update_by_struct(&task_uniform_buffer_data);
        
        encoder.set_global_uniform_buffer_view(self.uniform_view.clone());
        encoder.set_task_uniform_buffer_view(self.task_uniform_view.clone());

        let mut render_objects = context.render_objects.clone();
        for render_object in render_objects.iter_mut() {
            // 这个混在一起会不会有些奇怪。以后这类问题可以看看其他引擎的做法
            render_object
                .set_transform_view(mat_view)
                .set_transform_projection(mat_proj)
                .update_uniform_buffer();
        }

        encoder.encode_frame(|frame_encoder| {
            let swapchain_image_view = FTextureView::new_swapchain_view();

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

            self.pass2.set_color_attachments(vec![FAttachment::new(
                swapchain_image_view.clone(),
                ELoadOp::Load,
                EStoreOp::Store,
            )]);

            frame_encoder.encode_render_pass(&self.pass1, |pass_encoder| {
                // self.render_object1.encode(pass_encoder, "base");

                for render_object in render_objects.iter() {
                    render_object.encode(pass_encoder, "base");
                }
            });

            // frame_encoder.encode_render_pass(&self.pass2, |pass_encoder| {
            //     self.render_object2.encode(pass_encoder, "base");
            // });
        });

        encoder.present();
    }
}
