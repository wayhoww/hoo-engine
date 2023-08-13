use crate::device::graphics::*;
use crate::editor::importer::load_gltf_from_slice;
use crate::utils::*;
use crate::*;

use nalgebra_glm as glm;

pub struct FGraphicsPipeline {
    pass1: FPass,
    pass2: FPass,

    uniform_buffer: RcMut<FBuffer>,
    uniform_view: FBufferView,
    model: FModel,

    render_object1: FRenderObject,
    render_object2: FRenderObject,

    count: u64,
}

impl FGraphicsPipeline {
    pub async fn new_async<'a>(encoder: &FDeviceEncoder) -> Self {
        // material
        // let shader_code = "".to_string(); // get_text_from_url("/resources/main.wgsl").await.unwrap();
        let shader_code = r#"
// #define UNIFORM_BIND_GROUP_Material 0
// #define UNIFORM_BIND_GROUP_DrawCall 1
// #define UNIFORM_BIND_GROUP_Pass 2
// #define UNIFORM_BIND_GROUP_Task 3
// #define UNIFORM_BIND_GROUP_Global 4
// 有无 const 语法？

const xx = 0;

struct VertexOut {
    @builtin(position) position : vec4f,

    @location(0) uv0 : vec2f,
    @location(1) normal_local : vec3f
};

struct FragmentOut {
    @location(0) color : vec4f,
    // @builtin(frag_depth) depth: f32,
};

struct DrawCallUniform {
    transform_m: mat4x4f,
    transform_mv: mat4x4f,
    transform_mvp: mat4x4f,

    // color1: f32,
    // color2: f32,
};

@group(0) @binding(1) var<uniform> cDrawCall: DrawCallUniform;


@vertex
fn vsMain_base(
    @location(0) pos: vec3f, 
    @location(1) normal: vec3f, 
    @location(2) uv: vec2f
) -> VertexOut {
    // cDrawCall.matrix_projection * 
    var vertex_out: VertexOut;
    vertex_out.position = cDrawCall.transform_mvp * vec4f(pos.xyz, 1.0);
    vertex_out.uv0 = uv;
    vertex_out.normal_local = normal;
    
    return vertex_out;
}

@fragment
fn fsMain_base(vertex_out: VertexOut) -> FragmentOut {
    // w: linear depth
    // w/z: 0~1, 0 means near

    var fragment_out: FragmentOut;
    fragment_out.color = vec4f(abs(vertex_out.normal_local.xyz) + 0.3, 1.0);
    // fragment_out.depth = vertex_out.position.w / vertex_out.position.z;
    return fragment_out;
}
        "#;
        let material = rcmut!(FMaterial::new(shader_code.into()));
        material.borrow_mut().enable_shader_profile("base".into());
        material
            .borrow_mut()
            .enable_shader_profile("depthOnly".into());

        // mesh
        let load_result = load_gltf_from_slice(bundle::gltf_cube());

        let mesh = rcmut!(FMesh::from_file_resource(
            &load_result.unwrap()[0].sub_meshes[0]
        ));

        // model
        let model = FModel::new(mesh, material.clone());
        let render_object1 = FRenderObject::new(model.clone());
        let render_object2 = FRenderObject::new(model.clone());

        // faked
        let default_uniform_buffer = FBuffer::new_and_manage(BBufferUsages::Uniform);
        default_uniform_buffer.borrow_mut().resize(1024);
        let uniform_view = FBufferView::new_uniform(default_uniform_buffer.clone());

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
            uniform_buffer: default_uniform_buffer,
            uniform_view,
            model,
            render_object1,
            render_object2,
            count: 0,
        }
    }

    pub fn draw(&mut self, encoder: &mut FDeviceEncoder) {
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
            1000.0,
        );

        encoder.set_global_uniform_buffer_view(self.uniform_view.clone());

        self.render_object1
            .set_transform_model(mat_model)
            .set_transform_view(mat_view)
            .set_transform_projection(mat_proj)
            .update_uniform_buffer();

        self.render_object2
            .set_transform_model(mat_model_biased)
            .set_transform_view(mat_view)
            .set_transform_projection(mat_proj)
            .update_uniform_buffer();

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

            frame_encoder
                .get_device_encoder()
                .set_task_uniform_buffer_view(self.uniform_view.clone());

            frame_encoder.encode_render_pass(&self.pass1, |pass_encoder| {
                self.render_object1.encode(pass_encoder, "base");
            });

            frame_encoder.encode_render_pass(&self.pass2, |pass_encoder| {
                self.render_object2.encode(pass_encoder, "base");
            });
        });

        encoder.present();
    }
}
