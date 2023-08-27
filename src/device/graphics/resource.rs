use bitflags::bitflags;
use std::collections::HashMap;
use std::convert::TryFrom;
use wgpu;

use crate::io::resource::*;
use crate::utils::*;
use crate::*;

use super::utils::*;

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

impl From<BBufferUsages> for wgpu::BufferUsages {
    fn from(val: BBufferUsages) -> Self {
        let mut ret = wgpu::BufferUsages::empty();

        if val.contains(BBufferUsages::Vertex) {
            ret |= wgpu::BufferUsages::VERTEX;
        }
        if val.contains(BBufferUsages::Index) {
            ret |= wgpu::BufferUsages::INDEX;
        }
        if val.contains(BBufferUsages::Uniform) {
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
        false
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
            usages,
            device_buffer: None,

            data_updated: true,
            meta_updated: true,

            consolidation_id: 0,
        }
    }

    pub fn new_and_manage(usages: BBufferUsages) -> RcMut<Self> {
        let res = rcmut!(Self::new(usages));
        hoo_engine()
            .borrow()
            .get_resources()
            .add_gpu_resource(res.clone());
        res
    }

    pub fn update_by_array<T>(&mut self, data: &[T]) -> &mut Self {
        self.resize(std::mem::size_of_val(data) as u64);
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
            code,

            vertex_stage_entry: None,
            fragment_stage_entry: None,

            device_module: None,
            consolidation_id: 0,
        }
    }

    pub fn new_and_manage(code: String) -> RcMut<Self> {
        let res = rcmut!(Self::new(code));
        hoo_engine()
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

#[derive(Debug)]
pub struct FTexture {
    usages: BTextureUsages,
    format: ETextureFormat,
    width: u32,
    height: u32,
    device_texture: Option<wgpu::Texture>,

    updated: bool,

    consolidation_id: u64,
}

impl FTexture {
    fn new_internal(format: ETextureFormat, usages: BTextureUsages) -> Self {
        #[cfg(debug_assertions)]
        let _ = Into::<wgpu::TextureFormat>::into(format);

        Self {
            usages,
            format,
            width: 1,
            height: 1,
            device_texture: None,
            updated: false,
            consolidation_id: 0,
        }
    }

    pub fn new_and_manage(format: ETextureFormat, usages: BTextureUsages) -> RcMut<Self> {
        let res = rcmut!(Self::new_internal(format, usages));
        hoo_engine()
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

    pub fn get_format(&self) -> ETextureFormat {
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
            view_formats: &[Into::<wgpu::TextureFormat>::into(self.format)],
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
            FClearValue::Float4 {
                r: _,
                g: _,
                b: _,
                a: _,
            } => unimplemented!("float4 clear value"),
            FClearValue::Float(x) => x,
        })
    }
}

trait TValueFormat {
    fn block_size(&self) -> u32;
    fn dimension(&self) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ETextureFormat {
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgba16Float,
    Rgba32Float,
    R32Uint,
    Depth24PlusStencil8,
}

impl From<ETextureFormat> for wgpu::TextureFormat {
    fn from(val: ETextureFormat) -> Self {
        match val {
            ETextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            ETextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
            ETextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            ETextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
            ETextureFormat::R32Uint => wgpu::TextureFormat::R32Uint,
            ETextureFormat::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
        }
    }
}

impl TryFrom<wgpu::TextureFormat> for ETextureFormat {
    type Error = ();

    fn try_from(value: wgpu::TextureFormat) -> Result<Self, Self::Error> {
        match value {
            wgpu::TextureFormat::Bgra8Unorm => Ok(ETextureFormat::Bgra8Unorm),
            wgpu::TextureFormat::Bgra8UnormSrgb => Ok(ETextureFormat::Bgra8UnormSrgb),
            wgpu::TextureFormat::Rgba16Float => Ok(ETextureFormat::Rgba16Float),
            wgpu::TextureFormat::Rgba32Float => Ok(ETextureFormat::Rgba32Float),
            wgpu::TextureFormat::R32Uint => Ok(ETextureFormat::R32Uint),
            wgpu::TextureFormat::Depth24PlusStencil8 => Ok(ETextureFormat::Depth24PlusStencil8),
            _ => Err(()),
        }
    }
}

impl TValueFormat for ETextureFormat {
    fn block_size(&self) -> u32 {
        match self {
            ETextureFormat::Bgra8Unorm => 4,
            ETextureFormat::Bgra8UnormSrgb => 4,
            ETextureFormat::Rgba16Float => 8,
            ETextureFormat::Rgba32Float => 16,
            ETextureFormat::R32Uint => 4,
            ETextureFormat::Depth24PlusStencil8 => 4,
        }
    }

    fn dimension(&self) -> u32 {
        match self {
            ETextureFormat::Bgra8Unorm => 2,
            ETextureFormat::Bgra8UnormSrgb => 2,
            ETextureFormat::Rgba16Float => 2,
            ETextureFormat::Rgba32Float => 2,
            ETextureFormat::R32Uint => 1,
            ETextureFormat::Depth24PlusStencil8 => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EIndexFormat {
    Uint16,
    Uint32,
}

impl From<EIndexFormat> for wgpu::IndexFormat {
    fn from(val: EIndexFormat) -> Self {
        match val {
            EIndexFormat::Uint16 => wgpu::IndexFormat::Uint16,
            EIndexFormat::Uint32 => wgpu::IndexFormat::Uint32,
        }
    }
}

impl TryFrom<wgpu::IndexFormat> for EIndexFormat {
    type Error = ();

    fn try_from(value: wgpu::IndexFormat) -> Result<Self, Self::Error> {
        match value {
            wgpu::IndexFormat::Uint16 => Ok(EIndexFormat::Uint16),
            wgpu::IndexFormat::Uint32 => Ok(EIndexFormat::Uint32),
        }
    }
}

impl TValueFormat for EIndexFormat {
    fn block_size(&self) -> u32 {
        match self {
            EIndexFormat::Uint16 => 2,
            EIndexFormat::Uint32 => 4,
        }
    }

    fn dimension(&self) -> u32 {
        match self {
            EIndexFormat::Uint16 => 1,
            EIndexFormat::Uint32 => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EVertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
}

impl From<EVertexFormat> for wgpu::VertexFormat {
    fn from(val: EVertexFormat) -> Self {
        match val {
            EVertexFormat::Float32 => wgpu::VertexFormat::Float32,
            EVertexFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
            EVertexFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
            EVertexFormat::Float32x4 => wgpu::VertexFormat::Float32x4,
        }
    }
}

impl TryFrom<wgpu::VertexFormat> for EVertexFormat {
    type Error = ();

    fn try_from(value: wgpu::VertexFormat) -> Result<Self, Self::Error> {
        match value {
            wgpu::VertexFormat::Float32 => Ok(EVertexFormat::Float32),
            wgpu::VertexFormat::Float32x2 => Ok(EVertexFormat::Float32x2),
            wgpu::VertexFormat::Float32x3 => Ok(EVertexFormat::Float32x3),
            wgpu::VertexFormat::Float32x4 => Ok(EVertexFormat::Float32x4),
            _ => Err(()),
        }
    }
}

impl TValueFormat for EVertexFormat {
    fn block_size(&self) -> u32 {
        match self {
            EVertexFormat::Float32 => 4,
            EVertexFormat::Float32x2 => 8,
            EVertexFormat::Float32x3 => 12,
            EVertexFormat::Float32x4 => 16,
        }
    }

    fn dimension(&self) -> u32 {
        match self {
            EVertexFormat::Float32 => 1,
            EVertexFormat::Float32x2 => 2,
            EVertexFormat::Float32x3 => 3,
            EVertexFormat::Float32x4 => 4,
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

impl Default for FBufferView {
    fn default() -> Self {
        return FBufferView::new_uniform(FBuffer::new_and_manage(BBufferUsages::Uniform));
    }
}

impl FBufferView {
    pub fn new(buffer: RcMut<FBuffer>, offset: u64, size: u64, view_type: EBufferViewType) -> Self {
        let out = Self {
            buffer,
            offset,
            size,

            view_type,
        };

        debug_only!(out.check().unwrap());

        out
    }

    pub fn new_uniform(buffer: RcMut<FBuffer>) -> Self {
        let initial_size = buffer.borrow().size();
        Self::new(buffer, 0, initial_size, EBufferViewType::Uniform)
    }

    pub fn new_with_type(buffer: RcMut<FBuffer>, view_type: EBufferViewType) -> Self {
        let initial_size = buffer.borrow().size();
        Self::new(buffer, 0, initial_size, view_type)
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

        Ok(())
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

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BTextureUsages: u64 {
        const Attachment = 0x1;
        const Sampled = 0x4;
        const UnorderedAccess = 0x8;
    }
}

impl From<BTextureUsages> for wgpu::TextureUsages {
    fn from(val: BTextureUsages) -> Self {
        let mut res = wgpu::TextureUsages::empty();
        if val.contains(BTextureUsages::Attachment) {
            res |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }
        if val.contains(BTextureUsages::Sampled) {
            res |= wgpu::TextureUsages::TEXTURE_BINDING;
        }
        if val.contains(BTextureUsages::UnorderedAccess) {
            res |= wgpu::TextureUsages::STORAGE_BINDING;
        }
        res
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

    pub fn get_format(&self, encoder: &FDeviceEncoder) -> ETextureFormat {
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

        device_texture.create_view(&desc)
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

pub struct FDrawCommand {
    pub vertex_buffers: Vec<FVertexEntry>,
    pub index_buffer_view: FBufferView,
    pub index_count: u64,
    pub material_view: FBufferView,
    pub drawcall_view: FBufferView,
}

impl FDrawCommand {
    pub fn new(
        vertex_buffers: Vec<FVertexEntry>,
        index_buffer_view: FBufferView,
        index_count: u64,
        material_view: FBufferView,
        drawcall_view: FBufferView,
    ) -> Self {
        let out = Self {
            vertex_buffers,
            index_buffer_view,
            index_count,
            material_view,
            drawcall_view,
        };

        debug_only!(out.check().unwrap());

        out
    }

    pub fn check(&self) -> Result<(), String> {
        // check!(self.vertex_buffer_view.view_type == EBufferViewType::Vertex);
        check!(self.index_buffer_view.view_type == EBufferViewType::Index);
        check!(self.index_count * 4 <= self.index_buffer_view.size);
        // todo: check vertex buffer size

        Ok(())
    }

    pub fn get_index_buffer_view(&self) -> &FBufferView {
        &self.index_buffer_view
    }

    pub fn get_vertex_buffers(&self) -> &Vec<FVertexEntry> {
        &self.vertex_buffers
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

impl FClearValue {
    pub fn new_float4(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::Float4 { r, g, b, a }
    }

    pub fn new_float(x: f32) -> Self {
        Self::Float(x)
    }
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
            texture_view,
            load_op,
            store_op,
            clear_value: FClearValue::default(),
        }
    }

    pub fn new_append_to_view(view: FTextureView) -> Self {
        let attachment = Self::new(view, ELoadOp::Load, EStoreOp::Store);
        return attachment;
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

impl Default for FPass {
    fn default() -> Self {
        Self {
            uniform_view: FBufferView::new_uniform(FBuffer::new_and_manage(BBufferUsages::Uniform)),
            color_attachments: vec![],
            depth_stencil_attachment: None,
        }
    }
}

impl FPass {
    pub fn new(uniform_view: FBufferView) -> Self {
        Self {
            uniform_view,
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

#[derive(Clone)]
pub struct FVertexEntry {
    pub location: u32,
    pub format: EVertexFormat,
    pub view: FBufferView,
    pub offset: u64,
    pub stride: u64,
}

impl FVertexEntry {
    pub fn new_soa_entry(location: u32, view: FBufferView, format: EVertexFormat) -> Self {
        Self {
            location,
            format,
            view,
            offset: 0,
            stride: format.block_size() as u64,
        }
    }
}

#[derive(Clone)]
pub struct FMesh {
    vertex_buffers: Vec<FVertexEntry>,
    index_buffer_view: FBufferView,
    index_count: u64,
}

impl FMesh {
    pub fn new(
        vertex_buffers: Vec<FVertexEntry>,
        index_buffer_view: FBufferView,
        index_count: u64,
    ) -> Self {
        Self {
            vertex_buffers,
            index_buffer_view,
            index_count,
        }
    }

    pub fn get_vertex_buffers(&self) -> &Vec<FVertexEntry> {
        &self.vertex_buffers
    }

    pub fn get_index_buffer_view(&self) -> &FBufferView {
        &self.index_buffer_view
    }

    pub fn get_index_count(&self) -> u64 {
        self.index_count
    }

    fn make_buffer_from_slice<const R: usize>(slice: &[glm::TVec<f32, R>]) -> RcMut<FBuffer> {
        let mut vec: Vec<f32> = Vec::new();
        for v in slice.iter() {
            for i in 0..R {
                vec.push(v[i]);
            }
        }

        let buffer = FBuffer::new_and_manage(BBufferUsages::Vertex);

        {
            let mut buffer_ref = buffer.borrow_mut();
            buffer_ref.update_by_array(&vec);
        }

        buffer
    }

    pub fn from_file_resource(sub_mesh: &RSubMesh) -> Self {
        let vertex_position = Self::make_buffer_from_slice(&sub_mesh.positions);
        let vertex_normal = Self::make_buffer_from_slice(&sub_mesh.normals);
        let vertex_uv0 = Self::make_buffer_from_slice(&sub_mesh.uv0);

        let vertex_position_size = vertex_position.borrow().size();
        let vertex_normal_size = vertex_normal.borrow().size();
        let vertex_uv0_size = vertex_uv0.borrow().size();

        let view_position = FBufferView::new(
            vertex_position,
            0,
            vertex_position_size,
            EBufferViewType::Vertex,
        );
        let view_normal = FBufferView::new(
            vertex_normal,
            0,
            vertex_normal_size,
            EBufferViewType::Vertex,
        );
        let view_uv0 = FBufferView::new(vertex_uv0, 0, vertex_uv0_size, EBufferViewType::Vertex);

        let index_buffer = FBuffer::new_and_manage(BBufferUsages::Index);
        let index_buffer_size = sub_mesh.indices.len() * 4;
        index_buffer.borrow_mut().update_by_array(&sub_mesh.indices);
        let index_buffer_view = FBufferView::new(
            index_buffer,
            0,
            index_buffer_size as u64,
            EBufferViewType::Index,
        );

        let layout = vec![
            FVertexEntry::new_soa_entry(0, view_position, EVertexFormat::Float32x3),
            FVertexEntry::new_soa_entry(1, view_normal, EVertexFormat::Float32x3),
            FVertexEntry::new_soa_entry(2, view_uv0, EVertexFormat::Float32x2),
        ];

        Self::new(layout, index_buffer_view, sub_mesh.indices.len() as u64)
    }
}

// 暂时不搞 shader 变种，所以也没有 MaterialInstance
#[derive(Clone)]
pub struct FMaterial {
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
    pub fn new(shader: String) -> Self {
        // todo: reflection
        let buffer = FBuffer::new_and_manage(BBufferUsages::Uniform);
        let buffer_size = 16u64;
        buffer.borrow_mut().resize(buffer_size);
        let uniform_view = FBufferView::new(buffer, 0, buffer_size, EBufferViewType::Uniform);

        Self {
            shader_code: shader,
            shader_module: HashMap::new(),
            uniform_view,
        }
    }

    fn check_shader_profile_name(&self, name: &String) -> Result<(), String> {
        check!(regex::Regex::new(r"^[a-zA-Z_]+[a-zA-Z_0-9]*$")
            .unwrap()
            .is_match(name));

        Ok(())
    }

    pub fn enable_shader_profile(&mut self, name: String) -> &mut Self {
        if self.shader_module.contains_key(&name) {
            return self;
        }

        debug_only!(self.check_shader_profile_name(&name).unwrap());

        let shader_module = FShaderModule::new_and_manage(self.shader_code.clone());
        shader_module
            .borrow_mut()
            .set_vertex_stage_entry("vsMain_".to_owned() + &name);
        shader_module
            .borrow_mut()
            .set_fragment_stage_entry("fsMain_".to_owned() + &name);
        self.shader_module.insert(name, shader_module);
        self
    }

    pub fn update_uniform<T>(&mut self, data: &T) {
        self.uniform_view
            .get_buffer()
            .borrow_mut()
            .update_by_struct(data);
    }
}

pub trait TRenderObject {
    fn encode(&self, encoder: &mut FDeviceEncoder, shader_profile: &str);
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
    pub fn new(mesh: RcMut<FMesh>, material: RcMut<FMaterial>) -> Self {
        Self { mesh, material }
    }

    pub fn get_mesh(&self) -> RcMut<FMesh> {
        self.mesh.clone()
    }

    pub fn get_material(&self) -> RcMut<FMaterial> {
        self.material.clone()
    }
}

#[derive(Clone)]
pub struct FRenderObject {
    model: FModel,
    uniform_view: FBufferView,

    transform_m: glm::Mat4x4,
    transform_v: glm::Mat4x4,
    transform_p: glm::Mat4x4,
}

impl FRenderObject {
    pub fn new(model: FModel) -> Self {
        let uniform_buffer = FBuffer::new_and_manage(BBufferUsages::Uniform);

        let uniform_struct = DrawCallUniform::default();
        {
            uniform_buffer
                .borrow_mut()
                .update_by_struct(&uniform_struct);
        }
        let uniform_view = FBufferView::new_uniform(uniform_buffer);

        let mut out = Self {
            model,
            uniform_view,
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

    pub fn get_transform_model(&self) -> glm::Mat4x4 {
        self.transform_m
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

    pub fn encode(&self, encoder: &mut FPassEncoder, shader_profile: &str) {
        let mesh = self.model.mesh.borrow();
        let material = self.model.material.borrow();

        let cmd = FDrawCommand::new(
            mesh.get_vertex_buffers().clone(),
            mesh.get_index_buffer_view().clone(),
            mesh.get_index_count(),
            material.uniform_view.clone(),
            self.uniform_view.clone(),
        );

        encoder.setup_pipeline(
            mesh.get_vertex_buffers(),
            &material.get_shader_module(shader_profile).unwrap(),
        );
        encoder.draw(&cmd);
    }
}
