use crate::buffer::Texel;
use crate::pool::Pool;
use crate::run::{Execution, LaunchError};

/// Planned out and intrinsically validated command buffer.
///
/// This does not necessarily plan out a commands of low leve execution instruction set flavor.
/// This is selected based on the available device and its capabilities, which is performed during
/// launch.
pub struct Program {
    _todo: u8,
}

/// Low level instruction.
///
/// Can be scheduled/ran directly on a machine state. Our state machine is a simplified GL-like API
/// that fully manages lists of all created texture samples, shader modules, command buffers,
/// attachments, descriptors and passes.
///
/// Currently, resources are never deleted until the end of the program. All commands reference a
/// particular selected device/queue that is implicit global context.
pub enum Low {
    /// Create (and store) a render pipeline with specified parameters.
    RenderPipeline(RenderPipelineDescriptor),
    BindGroup(BindGroupDescriptor),

    /// Start a new command recording.  It reaches until `EndCommands` but can be interleaved with
    /// arbitrary other commands.
    BeginCommands,
    /// Starts a new render pass within the current command buffer, which can only contain render
    /// instructions. Has effect until `EndRenderPass`.
    BeginRenderPass(RenderPassDescriptor),
    /// Ends the command, push a new `CommandBuffer` to our list.
    EndCommands,
    /// End the render pass.
    EndRenderPass,
}

/// Create a bind group.
pub(crate) struct BindGroupDescriptor {
    /// Select the nth layout.
    layout_idx: usize,
    /// All entries at their natural position.
    entries: Vec<BindingResource>,
}

enum BindingResource {
    Buffer {
        buffer_idx: usize,
        offset: wgpu::BufferAddress,
        size: Option<wgpu::BufferSize>,
    },
    Sampler(usize),
    TextureView(usize),
}

/// Describe a bind group.
pub(crate) struct BindGroupLayoutDescriptor {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

/// Create a render pass.
pub(crate) struct RenderPassDescriptor {
    color_attachments: Vec<ColorAttachmentDescriptor>,
    depth_stencil: Option<DepthStencilDescriptor>,
}

struct ColorAttachmentDescriptor {
    texture_view: usize,
    ops: wgpu::Operations<wgpu::Color>,
}

struct DepthStencilDescriptor {
    texture_view: usize,
    depth_ops: Option<wgpu::Operations<f32>>,
    stencil_ops: Option<wgpu::Operations<u32>>,
}

/// The vertex+fragment shaders, primitive mode, layout and stencils.
/// Ignore multi sampling.
pub(crate) struct RenderPipelineDescriptor {
    layout: usize,
    vertex: VertexState,
    fragment: FragmentState,
}

struct VertexState {
    vertex_module: usize,
    entry_point: usize,
}

struct FragmentState {
    fragment_module: usize,
    entry_point: usize,
    targets: Vec<wgpu::ColorTargetState>,
}

/// For constructing a new buffer, of anonymous memory.
pub(crate) struct BufferDescriptor {
    size: wgpu::BufferAddress,
    usage: BufferUsage,
}

enum BufferUsage {
    /// Map Write + Vertex
    InVertices,
    /// Map Write + Storage + Copy Src
    DataIn,
    /// Map Read + Storage + Copy Dst
    DataOut,
    /// Map Read/Write + Storage + Copy Src/Dst
    DataInOut,
    /// Map Write + Uniform + Copy Src
    Uniform,
}

/// For constructing a new texture.
/// Ignores mip level, sample count, and some usages.
pub(crate) struct TextureDescriptor {
    size: (u32, u32),
    format: wgpu::TextureFormat,
    usage: TextureUsage,
}

enum TextureUsage {
    /// Copy Dst + Sampled
    DataIn,
    /// Copy Src + Render Attachment
    DataOut,
    /// A storage texture
    /// Copy Src/Dst + Sampled + Render Attachment
    Storage,
}

// FIXME: useless at the moment of writing, for our purposes.
// For reinterpreting parts of a texture.
// Ignores format (due to library restrictions), cube, aspect, mip level.
// pub(crate) struct TextureViewDescriptor;

/// For constructing a texture samples.
/// Ignores lod attributes
pub(crate) struct SamplerDescriptor {
    /// In all directions.
    address_mode: wgpu::AddressMode,
    resize_filter: wgpu::FilterMode,
    // TODO: evaluate if necessary or beneficial
    // compare: Option<wgpu::CompareFunction>,
    border_color: Option<wgpu::SamplerBorderColor>,
}

/// Cost planning data.
///
/// This helps quantify, approximate, or at least guess relative costs of operations with the goal
/// of supporting the planning of an execution plan. The internal unit of measurement is a copy of
/// one page of host memory to another page, based on the idea of directly expressing the costs for
/// a trivial pipeline with this.
pub struct CostModel {
    /// Do a 4×4 matrix multiplication on top of the copy.
    cpu_overhead_mul4x4: f32,
    /// Transfer a page to the default GPU.
    gpu_default_tx: f32,
    /// Transfer a page from the default GPU.
    gpu_default_rx: f32,
    /// Latency of scheduling something on the GPU.
    gpu_latency: f32,
}

/// The commands could not be made into a program.
pub enum CompileError {
}

/// Something won't work with this program and pool combination, no matter the amount of
/// configuration.
pub enum MismatchError {
}

/// Prepare program execution with a specific pool.
///
/// Some additional assembly and configuration might be required and possible. For example choose
/// specific devices for running, add push attributes,
pub struct Launcher<'program> {
    program: &'program Program,
    pool: &'program mut Pool,
}

impl Program {
    /// Run this program with a pool.
    ///
    /// Required input and output image descriptors must match those declared, or be convertible
    /// to them when a normalization operation was declared.
    pub fn launch<'pool>(&'pool self, pool: &'pool mut Pool)
        -> Result<Launcher<'pool>, MismatchError>
    {
        Ok(Launcher {
            program: self,
            pool,
        })
    }
}

impl Launcher<'_> {
    /// Really launch, potentially failing if configuration or inputs were missing etc.
    pub fn launch(self) -> Result<Execution, LaunchError> {
        todo!()
    }
}
