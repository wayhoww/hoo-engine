use bitflags::bitflags;
use std::collections::HashMap;
use web_sys::{GpuCanvasContext, GpuTextureDescriptor, GpuTextureFormat};

use crate::{
    check, debug_only, hoo_log,
    io::resource::RSubMesh,
    rcmut,
    renderer::utils::{jsarray, slice_to_bin_string},
    utils::types::RcMut,
    HooEngineRef, HooEngineWeak,
};

use super::{encoder::FWebGPUEncoder, utils::struct_to_bin_string};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BBufferUsages: u32 {
        const Vertex = 0x1;
        const Index = 0x2;
        const Uniform = 0x4;
    }
}

fn get_webgpu_buffer_usages(usages: BBufferUsages) -> u32 {
    let mut ret = 0;

    if usages.contains(BBufferUsages::Vertex) {
        ret |= web_sys::gpu_buffer_usage::VERTEX;
    }
    if usages.contains(BBufferUsages::Index) {
        ret |= web_sys::gpu_buffer_usage::INDEX;
    }
    if usages.contains(BBufferUsages::Uniform) {
        ret |= web_sys::gpu_buffer_usage::UNIFORM;
    }

    ret
}

pub trait TGPUResource {
    fn create_device_resource(&mut self, encoder: &mut FWebGPUEncoder);
    fn update_device_resource(&mut self, _: &mut FWebGPUEncoder) {}
    fn ready(&self) -> bool; // 有数据，但可能需要更新
    fn need_update(&self) -> bool {
        return false;
    } // 需要更新
}

pub struct FBuffer {
    data: Vec<u8>,
    usages: BBufferUsages,
    device_buffer: Option<web_sys::GpuBuffer>,

    data_updated: bool,
    meta_updated: bool,
}

impl FBuffer {
    // getter
    pub fn size(&self) -> u32 {
        self.data.len() as u32
    }

    // impl

    fn new(usages: BBufferUsages) -> Self {
        Self {
            data: vec![0; 4], // avoid empty buffer
            usages: usages,
            device_buffer: None,

            data_updated: true,
            meta_updated: true,
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
        self.resize((data.len() * std::mem::size_of::<T>()) as u32);
        self.data.copy_from_slice(slice_to_bin_string(data));
        self.data_updated = true;
        self
    }

    pub fn update_by_struct<T>(&mut self, data: &T) -> &mut Self {
        self.resize(std::mem::size_of::<T>() as u32);
        self.data.copy_from_slice(struct_to_bin_string(data));
        self.data_updated = true;
        self
    }

    pub fn resize(&mut self, size: u32) -> &mut Self {
        debug_assert!(size >= 4 && size % 4 == 0);
        self.data.resize(size as usize, 0);
        self.meta_updated = true;
        self
    }

    fn upload_data(&mut self, device: &web_sys::GpuDevice) {
        device.queue().write_buffer_with_u32_and_u8_array(
            self.device_buffer.as_ref().unwrap(),
            0,
            &self.data,
        );
        self.data_updated = false;
    }

    pub fn get_device_buffer(&self) -> &web_sys::GpuBuffer {
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
    fn update_device_resource(&mut self, encoder: &mut FWebGPUEncoder) {
        debug_assert!(self.device_buffer.is_some());

        if self.meta_updated {
            self.device_buffer.take().unwrap().destroy();
            self.create_device_resource(encoder);
        } else if self.data_updated {
            self.upload_data(encoder.get_device());
        }
    }

    fn create_device_resource(&mut self, encoder: &mut FWebGPUEncoder) {
        debug_assert!(self.device_buffer.is_none());
        debug_assert!(self.data.len() % 4 == 0);

        let descriptor = web_sys::GpuBufferDescriptor::new(
            self.data.len() as f64,
            web_sys::gpu_buffer_usage::COPY_DST | get_webgpu_buffer_usages(self.usages),
        );

        let device_buffer = encoder.get_device().create_buffer(&descriptor);
        self.device_buffer = Some(device_buffer);
        self.meta_updated = false;

        self.upload_data(encoder.get_device());
    }

    fn ready(&self) -> bool {
        self.device_buffer.is_some()
    }

    fn need_update(&self) -> bool {
        self.data_updated || self.meta_updated
    }
}

// 用于描述 buffer、texture 的数据格式。未必每个接口都支持所有格式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EValueFormat {
    Uint32,

    Float32,
    Float32x2,
    Float32x3,
    Float32x4,

    Unorm8x4,

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

            EValueFormat::Depth24Stencil8 => 32,
        }
    }
}

impl Into<GpuTextureFormat> for EValueFormat {
    fn into(self) -> GpuTextureFormat {
        match self {
            EValueFormat::Uint32 => GpuTextureFormat::R32uint,
            EValueFormat::Float32 => GpuTextureFormat::R32float,
            EValueFormat::Float32x2 => GpuTextureFormat::Rg32float,
            EValueFormat::Float32x3 => unimplemented!(),
            EValueFormat::Float32x4 => GpuTextureFormat::Rgba32float,
            EValueFormat::Unorm8x4 => GpuTextureFormat::Bgra8unorm,
            EValueFormat::Depth24Stencil8 => GpuTextureFormat::Depth24plusStencil8,
        }
    }
}

impl From<GpuTextureFormat> for EValueFormat {
    fn from(format: GpuTextureFormat) -> Self {
        match format {
            GpuTextureFormat::R32uint => EValueFormat::Uint32,
            GpuTextureFormat::R32float => EValueFormat::Float32,
            GpuTextureFormat::Rg32float => EValueFormat::Float32x2,
            GpuTextureFormat::Rgba32float => EValueFormat::Float32x4,
            GpuTextureFormat::Bgra8unorm => EValueFormat::Unorm8x4,
            GpuTextureFormat::Depth24plusStencil8 => EValueFormat::Depth24Stencil8,
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
    offset: u32,
    size: u32, // count, not size in bytes

    view_type: EBufferViewType,
}

impl FBufferView {
    pub fn new(buffer: RcMut<FBuffer>, offset: u32, size: u32, view_type: EBufferViewType) -> Self {
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

    pub fn get_offset(&self) -> u32 {
        self.offset
    }

    pub fn get_size(&self) -> u32 {
        debug_only!(self.check().unwrap());
        self.size
    }
}

pub struct FShaderModule {
    code: String,

    vertex_stage_entry: Option<String>,
    fragment_stage_entry: Option<String>,

    device_module: Option<web_sys::GpuShaderModule>,
}

impl FShaderModule {
    fn new(code: String) -> Self {
        Self {
            code: code,

            vertex_stage_entry: None,
            fragment_stage_entry: None,

            device_module: None,
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

    pub fn get_device_module(&self) -> Option<&web_sys::GpuShaderModule> {
        self.device_module.as_ref()
    }
}

impl TGPUResource for FShaderModule {
    fn create_device_resource(&mut self, encoder: &mut FWebGPUEncoder) {
        debug_assert!(self.device_module.is_none());

        let options = web_sys::GpuShaderModuleDescriptor::new(&self.code);
        self.device_module = Some(encoder.get_device().create_shader_module(&options));
    }

    fn ready(&self) -> bool {
        self.device_module.is_some()
    }

    fn need_update(&self) -> bool {
        false
    }
}

pub struct FDrawCommand {
    pub vertex_buffer_view: FBufferView,
    pub index_buffer_view: FBufferView,
    pub uniform_view: FBufferView,
    pub index_count: u32,
}

impl FDrawCommand {
    pub fn new(
        vertex_buffer_view: FBufferView,
        index_buffer_view: FBufferView,
        uniform_view: FBufferView,
        index_count: u32,
    ) -> Self {
        let out = Self {
            vertex_buffer_view: vertex_buffer_view,
            index_buffer_view: index_buffer_view,
            uniform_view: uniform_view,
            index_count: index_count,
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

    pub fn get_uniform_buffer_view(&self) -> &FBufferView {
        &self.uniform_view
    }

    pub fn get_index_count(&self) -> u32 {
        self.index_count
    }
}

pub const MAX_N_COLOR_ATTACHMENTS: usize = 4;

#[derive(Clone)]
pub struct FPass {
    uniform_view: FBufferView,

    color_attachments: [Option<FTextureView>; MAX_N_COLOR_ATTACHMENTS],
    depth_stencil_attachment: Option<FTextureView>,
}

impl FPass {
    pub fn new(uniform_view: FBufferView) -> Self {
        const NONE_VIEW: Option<FTextureView> = None;
        Self {
            uniform_view: uniform_view,
            color_attachments: [NONE_VIEW; MAX_N_COLOR_ATTACHMENTS],
            depth_stencil_attachment: None,
        }
    }

    pub fn set_depth_stencil_attachment(
        &mut self,
        depth_stencil_attachment: FTextureView,
    ) -> &mut Self {
        self.depth_stencil_attachment = Some(depth_stencil_attachment);
        self
    }

    pub fn clear_depth_stencil_attachment(&mut self) -> &mut Self {
        self.depth_stencil_attachment = None;
        self
    }

    pub fn get_depth_stencil_attachment(&self) -> Option<&FTextureView> {
        self.depth_stencil_attachment.as_ref()
    }

    pub fn set_color_attachments(
        &mut self,
        color_attachments: [Option<FTextureView>; MAX_N_COLOR_ATTACHMENTS],
    ) -> &mut Self {
        self.color_attachments = color_attachments;
        self
    }

    pub fn set_color_attachment(
        &mut self,
        index: usize,
        color_attachment: FTextureView,
    ) -> &mut Self {
        self.color_attachments[index] = Some(color_attachment);
        self
    }

    pub fn clear_color_attachments(&mut self) -> &mut Self {
        const NONE_VIEW: Option<FTextureView> = None;
        self.color_attachments = [NONE_VIEW; MAX_N_COLOR_ATTACHMENTS];
        self
    }

    pub fn get_color_attachments(&self) -> &[Option<FTextureView>; MAX_N_COLOR_ATTACHMENTS] {
        &self.color_attachments
    }

    pub fn get_uniform_buffer_view(&self) -> &FBufferView {
        &self.uniform_view
    }
}

pub struct FMesh {
    vertex_buffer_view: FBufferView,
    index_buffer_view: FBufferView,
    index_count: u32,
}

impl FMesh {
    pub fn new(
        vertex_buffer_view: FBufferView,
        index_buffer_view: FBufferView,
        index_count: u32,
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

    pub fn get_index_count(&self) -> u32 {
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
            vertex_buffer_size as u32,
            EBufferViewType::Vertex,
        );

        let index_buffer = FBuffer::new_and_manage(h, BBufferUsages::Index);
        let index_buffer_size = sub_mesh.indices.len() * 4;
        index_buffer.borrow_mut().update_by_array(&sub_mesh.indices);
        let index_buffer_view = FBufferView::new(
            index_buffer,
            0,
            index_buffer_size as u32,
            EBufferViewType::Index,
        );

        Self::new(
            vertex_buffer_view,
            index_buffer_view,
            sub_mesh.indices.len() as u32,
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
        let buffer_size = 16u32;
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
    fn encode(&self, encoder: &mut FWebGPUEncoder, pass_variant: &str);
}

pub struct FModel {
    mesh: RcMut<FMesh>,
    material: RcMut<FMaterial>,
}

impl FModel {
    pub fn new(_: HooEngineRef, mesh: RcMut<FMesh>, material: RcMut<FMaterial>) -> Self {
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

impl TRenderObject for FModel {
    fn encode(&self, encoder: &mut FWebGPUEncoder, pass_variant: &str) {
        let mesh = self.mesh.borrow();
        let material = self.material.borrow();

        let cmd = FDrawCommand::new(
            mesh.get_vertex_buffer_view().clone(),
            mesh.get_index_buffer_view().clone(),
            material.uniform_view.clone(),
            mesh.get_index_count(),
        );

        encoder.setup_pipeline(&material.get_shader_module(pass_variant).unwrap());
        encoder.draw(&cmd);
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BTextureUsages: u32 {
        const Attachment = 0x1;
        const Sampled = 0x4;
        const UnorderedAccess = 0x8;
    }
}

impl BTextureUsages {
    pub fn to_webgpu(&self) -> u32 {
        let mut res = 0u32;
        if self.contains(BTextureUsages::Attachment) {
            res |= web_sys::gpu_texture_usage::RENDER_ATTACHMENT;
        }
        if self.contains(BTextureUsages::Sampled) {
            res |= web_sys::gpu_texture_usage::TEXTURE_BINDING;
        }
        if self.contains(BTextureUsages::UnorderedAccess) {
            res |= web_sys::gpu_texture_usage::STORAGE_BINDING;
        }
        res
    }
}

pub struct FTexture {
    usages: BTextureUsages,
    format: EValueFormat,
    width: u32,
    height: u32,
    device_texture: Option<web_sys::GpuTexture>,

    updated: bool,
    is_swapchain: bool,
}

impl FTexture {
    fn new_internal(_: HooEngineRef, format: EValueFormat, usages: BTextureUsages) -> Self {
        #[cfg(debug_assertions)]
        let _ = Into::<web_sys::GpuTextureFormat>::into(format);

        Self {
            usages: usages,
            format: format,
            width: 1,
            height: 1,
            device_texture: None,
            updated: false,
            is_swapchain: false,
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

    pub fn new_swapchain_texture(_: HooEngineRef, context: &GpuCanvasContext) -> Self {
        let device_texture = context.get_current_texture();

        let width = device_texture.width();
        let height = device_texture.height();
        let format = device_texture.format();

        Self {
            usages: BTextureUsages::Attachment,
            format: format.into(),
            width: width,
            height: height,
            device_texture: Some(device_texture),
            updated: false,
            is_swapchain: true,
        }
    }

    pub fn new_swapchain_texture_and_manage(
        h: HooEngineRef,
        context: &GpuCanvasContext,
    ) -> RcMut<Self> {
        let res = rcmut!(Self::new_swapchain_texture(h, context));
        h.upgrade()
            .unwrap()
            .borrow()
            .get_resources()
            .add_gpu_resource(res.clone());
        res
    }

    pub fn update_swapchain_texture(&mut self, context: &GpuCanvasContext) {
        assert!(self.is_swapchain);

        let device_texture = context.get_current_texture();

        let width = device_texture.width();
        let height = device_texture.height();
        let format = device_texture.format();

        self.width = width;
        self.height = height;
        self.format = format.into();
        self.device_texture = Some(device_texture);
        self.updated = true;
    }

    pub fn clear_swapchain_texture(&mut self) {
        assert!(self.is_swapchain);
        self.device_texture = None;
    }

    pub fn set_width(&mut self, width: u32) -> &mut Self {
        assert!(!self.is_swapchain);

        self.width = width;
        self.updated = true;
        self
    }

    pub fn set_height(&mut self, height: u32) -> &mut Self {
        assert!(!self.is_swapchain);

        self.height = height;
        self.updated = true;
        self
    }

    pub fn set_size(&mut self, (width, height): (u32, u32)) -> &mut Self {
        assert!(!self.is_swapchain);

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

    pub fn get_device_texture(&self) -> &web_sys::GpuTexture {
        self.device_texture.as_ref().unwrap()
    }

    pub fn get_width(&self) -> u32 {
        if self.is_swapchain {
            return self.device_texture.as_ref().unwrap().width();
        }
        self.width
    }

    pub fn get_height(&self) -> u32 {
        if self.is_swapchain {
            return self.device_texture.as_ref().unwrap().height();
        }
        self.height
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.get_width(), self.get_height())
    }
}

impl TGPUResource for FTexture {
    fn create_device_resource(&mut self, encoder: &mut FWebGPUEncoder) {
        debug_assert!(self.device_texture.is_none());

        let device = encoder.get_device();
        let desc = GpuTextureDescriptor::new(
            self.format.into(),
            &jsarray([self.width, self.height, 1].as_slice()),
            self.usages.to_webgpu(),
        );
        self.device_texture = Some(device.create_texture(&desc));
        self.updated = false;
    }

    fn ready(&self) -> bool {
        self.device_texture.is_some()
    }

    fn update_device_resource(&mut self, encoder: &mut FWebGPUEncoder) {
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
}

impl Drop for FTexture {
    fn drop(&mut self) {
        if let Some(device_texture) = self.device_texture.as_ref() {
            device_texture.destroy();
        }
    }
}

#[derive(Clone)]
pub struct FTextureView {
    texture: RcMut<FTexture>,
}

impl FTextureView {
    pub fn new(texture: RcMut<FTexture>) -> Self {
        Self { texture: texture }
    }

    pub fn get_texture(&self) -> RcMut<FTexture> {
        self.texture.clone()
    }

    pub fn get_device_texture_view(&self) -> web_sys::GpuTextureView {
        self.texture.borrow_mut().get_device_texture().create_view()
    }
}
