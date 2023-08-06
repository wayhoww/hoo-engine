use bitflags::bitflags;
use std::{collections::HashMap, default};
use wgpu;

use crate::{
    check, debug_only, hoo_log, io::resource::RSubMesh, rcmut,
    renderer::utils::slice_to_bin_string, utils::types::RcMut, HooEngineRef, HooEngineWeak,
};

use super::{
    encoder::{FDeviceEncoder, FPassEncoder},
    utils::struct_to_bin_string,
};

use nalgebra_glm as glm;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BBufferUsages: u64 {
        const Vertex = 0x1;
        const Index = 0x2;
        const Uniform = 0x4;
    }
}

impl Into<wgpu::BufferUsages> for BBufferUsages {
    fn into(self) -> wgpu::BufferUsages {
        let mut ret = wgpu::BufferUsages::empty();

        if self.contains(BBufferUsages::Vertex) {
            ret |= wgpu::BufferUsages::VERTEX;
        }
        if self.contains(BBufferUsages::Index) {
            ret |= wgpu::BufferUsages::INDEX;
        }
        if self.contains(BBufferUsages::Uniform) {
            ret |= wgpu::BufferUsages::UNIFORM;
        }

        ret
    }
}

pub trait TGPUResource {
    fn create_device_resource(&mut self, encoder: &mut FDeviceEncoder);
    fn update_device_resource(&mut self, _: &mut FDeviceEncoder) {}
    fn ready(&self) -> bool; // 有数据，但可能需要更新
    fn need_update(&self) -> bool {
        return false;
    } // 需要更新

    fn assign_consolidation_id(&mut self, _: u64);
    fn get_consolidation_id(&self) -> u64;

    fn as_buffer(&self) -> Option<&FBuffer> {
        None
    }

    fn as_buffer_mut(&mut self) -> Option<&mut FBuffer> {
        None
    }

    fn as_texture(&self) -> Option<&FTexture> {
        None
    }

    fn as_texture_mut(&mut self) -> Option<&mut FTexture> {
        None
    }

    fn as_shader_module(&self) -> Option<&FShaderModule> {
        None
    }

    fn as_shader_module_mut(&mut self) -> Option<&mut FShaderModule> {
        None
    }
}

pub struct FBuffer {
    data: Vec<u8>,
    usages: BBufferUsages,
    device_buffer: Option<wgpu::Buffer>,

    data_updated: bool,
    meta_updated: bool,

    consolidation_id: u64,
}

impl FBuffer {
    // getter
    pub fn size(&self) -> u64 {
        self.data.len() as u64
    }

    // impl

    fn new(usages: BBufferUsages) -> Self {
        Self {
            data: vec![0; 4], // avoid empty buffer
            usages: usages,
            device_buffer: None,

            data_updated: true,
            meta_updated: true,

            consolidation_id: 0,
        }
    }

    pub fn new_and_manage(h: HooEngineRef, usages: BBufferUsages) -> RcMut<Self> {
        let res = rcmut!(Self::new(usages));
        h.upgrade()
            .unwrap()
            .borrow()
            .get_resources()
            .add_gpu_resource(res.clone());
        res
    }

    pub fn update_by_array<T>(&mut self, data: &[T]) -> &mut Self {
        self.resize((data.len() * std::mem::size_of::<T>()) as u64);
        self.data.copy_from_slice(slice_to_bin_string(data));
        self.data_updated = true;
        self
    }

    pub fn update_by_struct<T>(&mut self, data: &T) -> &mut Self {
        self.resize(std::mem::size_of::<T>() as u64);
        self.data.copy_from_slice(struct_to_bin_string(data));
        self.data_updated = true;
        self
    }

    pub fn resize(&mut self, size: u64) -> &mut Self {
        debug_assert!(size >= 4 && size % 4 == 0);
        self.data.resize(size as usize, 0);
        self.meta_updated = true;
        self
    }

    fn upload_data(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(self.device_buffer.as_ref().unwrap(), 0, &self.data);
        self.data_updated = false;
    }

    pub fn get_device_buffer(&self) -> &wgpu::Buffer {
        debug_assert!(self.device_buffer.is_some());
        self.device_buffer.as_ref().unwrap()
    }

    pub fn get_usages(&self) -> BBufferUsages {
        self.usages
    }
}

impl Drop for FBuffer {
    fn drop(&mut self) {
        if let Some(device_buffer) = self.device_buffer.as_ref() {
            device_buffer.destroy();
        }
    }
}

impl TGPUResource for FBuffer {
    fn update_device_resource(&mut self, encoder: &mut FDeviceEncoder) {
        debug_assert!(self.device_buffer.is_some());

        if self.meta_updated {
            self.device_buffer.take().unwrap().destroy();
            self.create_device_resource(encoder);
        } else if self.data_updated {
            self.upload_data(encoder.get_queue());
        }
    }

    fn create_device_resource(&mut self, encoder: &mut FDeviceEncoder) {
        debug_assert!(self.device_buffer.is_none());
        debug_assert!(self.data.len() % 4 == 0);

        let descriptor = wgpu::BufferDescriptor {
            label: None,
            size: self.data.len() as u64,
            usage: wgpu::BufferUsages::COPY_DST | self.usages.into(),
            mapped_at_creation: false,
        };

        let device_buffer = encoder.get_device().create_buffer(&descriptor);
        self.device_buffer = Some(device_buffer);
        self.meta_updated = false;

        self.upload_data(encoder.get_queue());
    }

    fn ready(&self) -> bool {
        self.device_buffer.is_some()
    }

    fn need_update(&self) -> bool {
        self.data_updated || self.meta_updated
    }

    fn assign_consolidation_id(&mut self, id: u64) {
        self.consolidation_id = id;
    }

    fn get_consolidation_id(&self) -> u64 {
        self.consolidation_id
    }

    fn as_buffer(&self) -> Option<&FBuffer> {
        Some(self)
    }

    fn as_buffer_mut(&mut self) -> Option<&mut FBuffer> {
        Some(self)
    }
}

// TODO: 所有 buffer 类型，附加上 encoder，放进 backend 里
// 其他放进 renderer 里面

// TODO：这个，拆！
// 用于描述 buffer、texture 的数据格式。未必每个接口都支持所有格式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EValueFormat {
    Uint32,

    Float32,
    Float32x2,
    Float32x3,
    Float32x4,

    Unorm8x4,
    Unorm8x4Srgb,

    Depth24Stencil8,
}

impl EValueFormat {
    pub fn size(&self) -> usize {
        match self {
            EValueFormat::Uint32 => 4,

            EValueFormat::Float32 => 4,
            EValueFormat::Float32x2 => 8,
            EValueFormat::Float32x3 => 12,
            EValueFormat::Float32x4 => 16,

            EValueFormat::Unorm8x4 => 4,
            EValueFormat::Unorm8x4Srgb => 4,

            EValueFormat::Depth24Stencil8 => 32,
        }
    }
}

impl Into<wgpu::TextureFormat> for EValueFormat {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            EValueFormat::Uint32 => wgpu::TextureFormat::R32Uint,
            EValueFormat::Float32 => wgpu::TextureFormat::R32Float,
            EValueFormat::Float32x2 => wgpu::TextureFormat::Rg32Float,
            EValueFormat::Float32x3 => unimplemented!(),
            EValueFormat::Float32x4 => wgpu::TextureFormat::Rgba32Float,
            EValueFormat::Unorm8x4 => wgpu::TextureFormat::Bgra8Unorm,
            EValueFormat::Unorm8x4Srgb => wgpu::TextureFormat::Bgra8UnormSrgb,
            EValueFormat::Depth24Stencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
        }
    }
}

impl From<wgpu::TextureFormat> for EValueFormat {
    fn from(format: wgpu::TextureFormat) -> Self {
        match format {
            wgpu::TextureFormat::R32Uint => EValueFormat::Uint32,
            wgpu::TextureFormat::R32Float => EValueFormat::Float32,
            wgpu::TextureFormat::Rg32Float => EValueFormat::Float32x2,
            wgpu::TextureFormat::Rgba32Float => EValueFormat::Float32x4,
            wgpu::TextureFormat::Bgra8Unorm => EValueFormat::Unorm8x4,
            wgpu::TextureFormat::Bgra8UnormSrgb => EValueFormat::Unorm8x4Srgb,
            wgpu::TextureFormat::Depth24PlusStencil8 => EValueFormat::Depth24Stencil8,
            format => unimplemented!("{:?}", format),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EBufferViewType {
    Vertex,
    Index,
    Uniform,
}

impl EBufferViewType {
    fn required_usage(&self) -> BBufferUsages {
        match self {
            EBufferViewType::Vertex => BBufferUsages::Vertex,
            EBufferViewType::Index => BBufferUsages::Index,
            EBufferViewType::Uniform => BBufferUsages::Uniform,
        }
    }
}

#[derive(Clone)]
pub struct FBufferView {
    buffer: RcMut<FBuffer>,
    offset: u64,
    size: u64, // count, not size in bytes

    view_type: EBufferViewType,
}

impl FBufferView {
    pub fn new(buffer: RcMut<FBuffer>, offset: u64, size: u64, view_type: EBufferViewType) -> Self {
        let out = Self {
            buffer: buffer,
            offset: offset,
            size: size,

            view_type: view_type,
        };

        debug_only!(out.check().unwrap());

        out
    }

    pub fn new_uniform(buffer: RcMut<FBuffer>) -> Self {
        let initial_size = buffer.borrow().size();
        Self::new(buffer, 0, initial_size, EBufferViewType::Uniform)
    }

    pub fn check(&self) -> Result<(), String> {
        // count
        check!(self.offset as usize + self.size as usize <= self.buffer.borrow().data.len());

        // usage
        check!(self
            .buffer
            .borrow()
            .usages
            .contains(self.view_type.required_usage()));

        return Ok(());
    }

    pub fn get_buffer(&self) -> RcMut<FBuffer> {
        self.buffer.clone()
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u64 {
        debug_only!(self.check().unwrap());
        self.size
    }
}

pub struct FShaderModule {
    code: String,

    vertex_stage_entry: Option<String>,
    fragment_stage_entry: Option<String>,

    device_module: Option<wgpu::ShaderModule>,

    consolidation_id: u64,
}

impl FShaderModule {
    fn new(code: String) -> Self {
        Self {
            code: code,

            vertex_stage_entry: None,
            fragment_stage_entry: None,

            device_module: None,
            consolidation_id: 0,
        }
    }

    pub fn new_and_manage(h: HooEngineRef, code: String) -> RcMut<Self> {
        let res = rcmut!(Self::new(code));
        h.upgrade()
            .unwrap()
            .borrow()
            .get_resources()
            .add_gpu_resource(res.clone());
        res
    }

    pub fn set_vertex_stage_entry(&mut self, entry: String) -> &mut Self {
        self.vertex_stage_entry = Some(entry);
        self
    }

    pub fn set_fragment_stage_entry(&mut self, entry: String) -> &mut Self {
        self.fragment_stage_entry = Some(entry);
        self
    }

    pub fn get_vertex_stage_entry(&self) -> Option<&String> {
        self.vertex_stage_entry.as_ref()
    }

    pub fn get_fragment_stage_entry(&self) -> Option<&String> {
        self.fragment_stage_entry.as_ref()
    }

    pub fn get_device_module(&self) -> Option<&wgpu::ShaderModule> {
        self.device_module.as_ref()
    }
}

impl TGPUResource for FShaderModule {
    fn create_device_resource(&mut self, encoder: &mut FDeviceEncoder) {
        debug_assert!(self.device_module.is_none());

        let options = wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(self.code.clone().into()),
        };
        self.device_module = Some(encoder.get_device().create_shader_module(options));
    }

    fn ready(&self) -> bool {
        self.device_module.is_some()
    }

    fn need_update(&self) -> bool {
        false
    }

    fn assign_consolidation_id(&mut self, id: u64) {
        self.consolidation_id = id;
    }

    fn get_consolidation_id(&self) -> u64 {
        self.consolidation_id
    }

    fn as_shader_module(&self) -> Option<&FShaderModule> {
        Some(self)
    }

    fn as_shader_module_mut(&mut self) -> Option<&mut FShaderModule> {
        Some(self)
    }
}

pub struct FDrawCommand {
    pub vertex_buffer_view: FBufferView,
    pub index_buffer_view: FBufferView,
    pub index_count: u64,
    pub material_view: FBufferView,
    pub drawcall_view: FBufferView,
}

impl FDrawCommand {
    pub fn new(
        vertex_buffer_view: FBufferView,
        index_buffer_view: FBufferView,
        index_count: u64,
        material_view: FBufferView,
        drawcall_view: FBufferView,
    ) -> Self {
        let out = Self {
            vertex_buffer_view: vertex_buffer_view,
            index_buffer_view: index_buffer_view,
            index_count: index_count,
            material_view: material_view,
            drawcall_view: drawcall_view,
        };

        debug_only!(out.check().unwrap());

        out
    }

    pub fn check(&self) -> Result<(), String> {
        check!(self.vertex_buffer_view.view_type == EBufferViewType::Vertex);
        check!(self.index_buffer_view.view_type == EBufferViewType::Index);
        check!(self.index_count * 4 <= self.index_buffer_view.size);
        // todo: check vertex buffer size

        return Ok(());
    }

    pub fn get_index_buffer_view(&self) -> &FBufferView {
        &self.index_buffer_view
    }

    pub fn get_vertex_buffer_view(&self) -> &FBufferView {
        &self.vertex_buffer_view
    }

    pub fn get_material_view(&self) -> &FBufferView {
        &self.material_view
    }

    pub fn get_drawcall_view(&self) -> &FBufferView {
        &self.drawcall_view
    }

    pub fn get_index_count(&self) -> u64 {
        self.index_count
    }
}

pub const MAX_N_COLOR_ATTACHMENTS: usize = 4;

#[derive(Clone, Debug)]
pub enum ELoadOp {
    Load,
    Clear,
}

impl ELoadOp {
    pub fn to_wgpu<V: Clone>(&self, c: &V) -> wgpu::LoadOp<V> {
        match self {
            ELoadOp::Load => wgpu::LoadOp::Load,
            ELoadOp::Clear => wgpu::LoadOp::Clear(c.clone()),
        }
    }

    pub fn to_wgpu_color(&self, clear_val: &FClearValue) -> wgpu::LoadOp<wgpu::Color> {
        self.to_wgpu(&match clear_val.clone() {
            FClearValue::Zero => wgpu::Color::BLACK,
            FClearValue::Float4 { r, g, b, a } => wgpu::Color {
                r: r as f64,
                g: g as f64,
                b: b as f64,
                a: a as f64,
            },
            FClearValue::Float(x) => wgpu::Color {
                r: x as f64,
                g: x as f64,
                b: x as f64,
                a: x as f64,
            },
        })
    }

    pub fn to_wgpu_value(&self, clear_val: FClearValue) -> wgpu::LoadOp<f32> {
        self.to_wgpu(&match clear_val {
            FClearValue::Zero => 0.0,
            FClearValue::Float4 { r, g, b, a } => unimplemented!("float4 clear value"),
            FClearValue::Float(x) => x,
        })
    }
}

#[derive(Clone, Debug)]
pub enum EStoreOp {
    Store,
    Discard,
}

impl EStoreOp {
    pub fn store(&self) -> bool {
        match self {
            EStoreOp::Store => true,
            EStoreOp::Discard => false,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub enum FClearValue {
    #[default]
    Zero,
    Float4 {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
    Float(f32),
}

#[derive(Clone, Debug)]
pub struct FAttachment {
    pub texture_view: FTextureView,
    pub load_op: ELoadOp,
    pub store_op: EStoreOp,
    pub clear_value: FClearValue,
}

impl FAttachment {
    pub fn new(texture_view: FTextureView, load_op: ELoadOp, store_op: EStoreOp) -> Self {
        Self {
            texture_view: texture_view,
            load_op: load_op,
            store_op: store_op,
            clear_value: FClearValue::default(),
        }
    }

    pub fn set_clear_value(&mut self, clear_value: FClearValue) -> &mut Self {
        // check: clear value vs texture format
        self.clear_value = clear_value;
        self
    }
}

#[derive(Clone)]
pub struct FPass {
    uniform_view: FBufferView,

    color_attachments: Vec<FAttachment>,
    depth_stencil_attachment: Option<FAttachment>,
}

impl FPass {
    pub fn new(uniform_view: FBufferView) -> Self {
        const NONE_VIEW: Option<FTextureView> = None;
        Self {
            uniform_view: uniform_view,
            color_attachments: vec![],
            depth_stencil_attachment: None,
        }
    }

    pub fn set_depth_stencil_attachment(
        &mut self,
        depth_stencil_attachment: FAttachment,
    ) -> &mut Self {
        self.depth_stencil_attachment = Some(depth_stencil_attachment);
        self
    }

    pub fn clear_depth_stencil_attachment(&mut self) -> &mut Self {
        self.depth_stencil_attachment = None;
        self
    }

    pub fn get_depth_stencil_attachment(&self) -> &Option<FAttachment> {
        &self.depth_stencil_attachment
    }

    pub fn set_color_attachments(&mut self, color_attachments: Vec<FAttachment>) -> &mut Self {
        debug_assert!(color_attachments.len() <= MAX_N_COLOR_ATTACHMENTS);
        self.color_attachments = color_attachments;
        self
    }

    pub fn get_color_attachments(&self) -> &Vec<FAttachment> {
        &self.color_attachments
    }

    pub fn get_uniform_buffer_view(&self) -> &FBufferView {
        &self.uniform_view
    }
}

pub struct FMesh {
    vertex_buffer_view: FBufferView,
    index_buffer_view: FBufferView,
    index_count: u64,
}

impl FMesh {
    pub fn new(
        vertex_buffer_view: FBufferView,
        index_buffer_view: FBufferView,
        index_count: u64,
    ) -> Self {
        Self {
            vertex_buffer_view: vertex_buffer_view,
            index_buffer_view: index_buffer_view,
            index_count: index_count,
        }
    }

    pub fn get_vertex_buffer_view(&self) -> &FBufferView {
        &self.vertex_buffer_view
    }

    pub fn get_index_buffer_view(&self) -> &FBufferView {
        &self.index_buffer_view
    }

    pub fn get_index_count(&self) -> u64 {
        self.index_count
    }

    pub fn from_file_resource(h: HooEngineRef, sub_mesh: &RSubMesh) -> Self {
        let vertex_buffer = FBuffer::new_and_manage(h, BBufferUsages::Vertex);
        let vertex_buffer_size = sub_mesh.positions.len() * (3 + 3 + 2) * 4;

        {
            let mut vertex_buffer_ref = vertex_buffer.borrow_mut();

            let mut data = Vec::new();
            for ((pos, normal), uv) in sub_mesh
                .positions
                .iter()
                .zip(sub_mesh.normals.iter())
                .zip(sub_mesh.uv0.iter())
            {
                data.push(pos[0]);
                data.push(pos[1]);
                data.push(pos[2]);

                data.push(normal[0]);
                data.push(normal[1]);
                data.push(normal[2]);

                data.push(uv[0]);
                data.push(uv[1]);
            }

            vertex_buffer_ref.update_by_array(&data);
        }

        let vertex_buffer_view = FBufferView::new(
            vertex_buffer,
            0,
            vertex_buffer_size as u64,
            EBufferViewType::Vertex,
        );

        let index_buffer = FBuffer::new_and_manage(h, BBufferUsages::Index);
        let index_buffer_size = sub_mesh.indices.len() * 4;
        index_buffer.borrow_mut().update_by_array(&sub_mesh.indices);
        let index_buffer_view = FBufferView::new(
            index_buffer,
            0,
            index_buffer_size as u64,
            EBufferViewType::Index,
        );

        Self::new(
            vertex_buffer_view,
            index_buffer_view,
            sub_mesh.indices.len() as u64,
        )
    }
}

// 暂时不搞 shader 变种，所以也没有 MaterialInstance
pub struct FMaterial {
    hoo_engine: HooEngineWeak,

    shader_code: String,
    shader_module: HashMap<String, RcMut<FShaderModule>>,
    uniform_view: FBufferView,
}

impl FMaterial {
    // getter
    pub fn get_shader_module(&self, name: &str) -> Option<RcMut<FShaderModule>> {
        self.shader_module.get(name).map(Clone::clone)
    }

    // impl
    pub fn new(h: HooEngineRef, shader: String) -> Self {
        // todo: reflection
        let buffer = FBuffer::new_and_manage(h, BBufferUsages::Uniform);
        let buffer_size = 16u64;
        buffer.borrow_mut().resize(buffer_size);
        let uniform_view = FBufferView::new(buffer, 0, buffer_size, EBufferViewType::Uniform);

        Self {
            hoo_engine: HooEngineWeak::from(h.clone()),
            shader_code: shader,
            shader_module: HashMap::new(),
            uniform_view: uniform_view,
        }
    }

    fn check_pass_variant_name(&self, name: &String) -> Result<(), String> {
        check!(regex::Regex::new(r"^[a-zA-Z_]+[a-zA-Z_0-9]*$")
            .unwrap()
            .is_match(name));

        return Ok(());
    }

    // TODO: variant 是个形容词
    pub fn enable_pass_variant(&mut self, name: String) -> &mut Self {
        if self.shader_module.contains_key(&name) {
            return self;
        }

        debug_only!(self.check_pass_variant_name(&name).unwrap());

        let shader_module =
            FShaderModule::new_and_manage(&self.hoo_engine, self.shader_code.clone());
        shader_module
            .borrow_mut()
            .set_vertex_stage_entry("vsMain_".to_owned() + &name);
        shader_module
            .borrow_mut()
            .set_fragment_stage_entry("fsMain_".to_owned() + &name);
        self.shader_module.insert(name, shader_module);
        return self;
    }

    pub fn update_uniform<T>(&mut self, data: &T) {
        self.uniform_view
            .get_buffer()
            .borrow_mut()
            .update_by_struct(data);
    }
}

pub trait TRenderObject {
    fn encode(&self, encoder: &mut FDeviceEncoder, pass_variant: &str);
}

#[repr(packed)]
struct DrawCallUniform {
    transform_m: glm::Mat4x4,
    transform_mv: glm::Mat4x4,
    transform_mvp: glm::Mat4x4,
}

impl Default for DrawCallUniform {
    fn default() -> Self {
        Self {
            transform_m: glm::identity(),
            transform_mv: glm::identity(),
            transform_mvp: glm::identity(),
        }
    }
}

#[derive(Clone)]
pub struct FModel {
    mesh: RcMut<FMesh>,
    material: RcMut<FMaterial>,
}

impl FModel {
    pub fn new(h: HooEngineRef, mesh: RcMut<FMesh>, material: RcMut<FMaterial>) -> Self {
        Self {
            mesh: mesh,
            material: material,
        }
    }

    pub fn get_mesh(&self) -> RcMut<FMesh> {
        self.mesh.clone()
    }

    pub fn get_material(&self) -> RcMut<FMaterial> {
        self.material.clone()
    }
}

pub struct FRenderObject {
    model: FModel,
    uniform_view: FBufferView,

    transform_m: glm::Mat4x4,
    transform_v: glm::Mat4x4,
    transform_p: glm::Mat4x4,
}

impl FRenderObject {
    pub fn new(h: HooEngineRef, model: FModel) -> Self {
        let uniform_buffer = FBuffer::new_and_manage(h, BBufferUsages::Uniform);

        let uniform_struct = DrawCallUniform::default();
        {
            uniform_buffer
                .borrow_mut()
                .update_by_struct(&uniform_struct);
        }
        let uniform_view = FBufferView::new_uniform(uniform_buffer);

        let mut out = Self {
            model: model,
            uniform_view: uniform_view,
            transform_m: glm::identity(),
            transform_v: glm::identity(),
            transform_p: glm::identity(),
        };

        out.update_uniform_buffer();

        out
    }

    pub fn set_transform_model(&mut self, transform: glm::Mat4x4) -> &mut Self {
        self.transform_m = transform;
        self
    }

    pub fn set_transform_view(&mut self, transform: glm::Mat4x4) -> &mut Self {
        self.transform_v = transform;
        self
    }

    pub fn set_transform_projection(&mut self, transform: glm::Mat4x4) -> &mut Self {
        self.transform_p = transform;
        self
    }

    pub fn update_uniform_buffer(&mut self) {
        let mut uniform_struct = DrawCallUniform::default();
        uniform_struct.transform_m = self.transform_m;
        uniform_struct.transform_mv = self.transform_v * self.transform_m;
        uniform_struct.transform_mvp = self.transform_p * self.transform_v * self.transform_m;
        self.uniform_view
            .get_buffer()
            .borrow_mut()
            .update_by_struct(&uniform_struct);
    }

    pub fn encode(&self, encoder: &mut FPassEncoder, pass_variant: &str) {
        let mesh = self.model.mesh.borrow();
        let material = self.model.material.borrow();

        let cmd = FDrawCommand::new(
            mesh.get_vertex_buffer_view().clone(),
            mesh.get_index_buffer_view().clone(),
            mesh.get_index_count(),
            material.uniform_view.clone(),
            self.uniform_view.clone(),
        );

        encoder.setup_pipeline(&material.get_shader_module(pass_variant).unwrap());
        encoder.draw(&cmd);
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BTextureUsages: u64 {
        const Attachment = 0x1;
        const Sampled = 0x4;
        const UnorderedAccess = 0x8;
    }
}

impl Into<wgpu::TextureUsages> for BTextureUsages {
    fn into(self) -> wgpu::TextureUsages {
        let mut res = wgpu::TextureUsages::empty();
        if self.contains(BTextureUsages::Attachment) {
            res |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }
        if self.contains(BTextureUsages::Sampled) {
            res |= wgpu::TextureUsages::TEXTURE_BINDING;
        }
        if self.contains(BTextureUsages::UnorderedAccess) {
            res |= wgpu::TextureUsages::STORAGE_BINDING;
        }
        res
    }
}

#[derive(Debug)]
pub struct FTexture {
    usages: BTextureUsages,
    format: EValueFormat,
    width: u32,
    height: u32,
    device_texture: Option<wgpu::Texture>,

    updated: bool,

    consolidation_id: u64,
}

impl FTexture {
    fn new_internal(_: HooEngineRef, format: EValueFormat, usages: BTextureUsages) -> Self {
        #[cfg(debug_assertions)]
        let _ = Into::<wgpu::TextureFormat>::into(format);

        Self {
            usages: usages,
            format: format,
            width: 1,
            height: 1,
            device_texture: None,
            updated: false,
            consolidation_id: 0,
        }
    }

    pub fn new_and_manage(
        h: HooEngineRef,
        format: EValueFormat,
        usages: BTextureUsages,
    ) -> RcMut<Self> {
        let res = rcmut!(Self::new_internal(h, format, usages));
        h.upgrade()
            .unwrap()
            .borrow()
            .get_resources()
            .add_gpu_resource(res.clone());
        res
    }

    pub fn set_width(&mut self, width: u32) -> &mut Self {
        self.width = width;
        self.updated = true;
        self
    }

    pub fn set_height(&mut self, height: u32) -> &mut Self {
        self.height = height;
        self.updated = true;
        self
    }

    pub fn set_size(&mut self, (width, height): (u32, u32)) -> &mut Self {
        self.width = width;
        self.height = height;
        self.updated = true;
        self
    }

    pub fn get_usages(&self) -> BTextureUsages {
        self.usages
    }

    pub fn get_format(&self) -> EValueFormat {
        self.format
    }

    pub fn get_device_texture(&self) -> &wgpu::Texture {
        self.device_texture.as_ref().unwrap()
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.get_width(), self.get_height())
    }
}

impl TGPUResource for FTexture {
    fn create_device_resource(&mut self, encoder: &mut FDeviceEncoder) {
        debug_assert!(self.device_texture.is_none());

        let device = encoder.get_device();
        let desc = wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            format: self.format.into(),
            dimension: wgpu::TextureDimension::D2,
            usage: self.usages.into(),
            view_formats: &[Into::<wgpu::TextureFormat>::into(self.format)], // TODO：还可以允许对应的 SRGB
        };

        self.device_texture = Some(device.create_texture(&desc));
        self.updated = false;
    }

    fn ready(&self) -> bool {
        self.device_texture.is_some()
    }

    fn update_device_resource(&mut self, encoder: &mut FDeviceEncoder) {
        debug_assert!(self.device_texture.is_some());
        if !self.updated {
            return;
        }
        self.device_texture.take().unwrap().destroy();
        self.create_device_resource(encoder);
        self.updated = false;
    }

    fn need_update(&self) -> bool {
        self.updated
    }

    fn assign_consolidation_id(&mut self, id: u64) {
        self.consolidation_id = id;
    }

    fn get_consolidation_id(&self) -> u64 {
        self.consolidation_id
    }

    fn as_texture(&self) -> Option<&FTexture> {
        Some(self)
    }

    fn as_texture_mut(&mut self) -> Option<&mut FTexture> {
        Some(self)
    }
}

impl Drop for FTexture {
    fn drop(&mut self) {
        if let Some(device_texture) = self.device_texture.as_ref() {
            device_texture.destroy();
        }
    }
}

#[derive(Clone, Debug)]
enum FTextureViewType {
    Texture(RcMut<FTexture>),
    SwapChain,
}

#[derive(Clone, Debug)]
pub struct FTextureView {
    texture: FTextureViewType,
}

impl FTextureView {
    pub fn new(texture: RcMut<FTexture>) -> Self {
        Self {
            texture: FTextureViewType::Texture(texture),
        }
    }

    pub fn new_swapchain_view() -> Self {
        Self {
            texture: FTextureViewType::SwapChain,
        }
    }

    pub fn get_texture(&self) -> RcMut<FTexture> {
        match &self.texture {
            FTextureViewType::Texture(texture) => texture.clone(),
            FTextureViewType::SwapChain => panic!("Texture is a SwapChain."),
        }
    }

    pub fn get_format(&self, encoder: &FDeviceEncoder) -> EValueFormat {
        match &self.texture {
            FTextureViewType::Texture(texture) => texture.borrow().get_format(),
            FTextureViewType::SwapChain => encoder.get_swapchain_format(),
        }
    }

    fn create_device_texture_view(&self, device_texture: &wgpu::Texture) -> wgpu::TextureView {
        // 要么做成 GPUResource，要么就不要直接返回 device 资源。倾向于 device
        let desc = wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All, // all / depth / stencil
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        };

        return device_texture.create_view(&desc);
    }

    pub fn get_device_texture_view(&self, encoder: &FDeviceEncoder) -> wgpu::TextureView {
        match &self.texture {
            FTextureViewType::Texture(texture) => {
                self.create_device_texture_view(texture.borrow().get_device_texture())
            }
            FTextureViewType::SwapChain => {
                let surf = encoder.get_swapchain_texture();
                let tex = &surf.as_ref().unwrap().texture;
                self.create_device_texture_view(tex)
            }
        }
    }
}
