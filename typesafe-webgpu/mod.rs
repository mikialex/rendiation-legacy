use wgpu::BufferAddress;

use super::{buffer::*, If, True, True2};

pub struct GPUCommandEncoder {
  encoder: wgpu::CommandEncoder,
}

impl GPUCommandEncoder {
  pub fn copy_buffer_to_buffer<
    const DST_USAGE: wgpu::BufferUsages,
    const SRC_USAGE: wgpu::BufferUsages,
  >(
    &mut self,
    source: &Buffer<SRC_USAGE>,
    source_offset: BufferAddress,
    destination: &Buffer<DST_USAGE>,
    destination_offset: BufferAddress,
    copy_size: BufferAddress,
  ) where
    If<{ has_copy_src(SRC_USAGE) }>: True,
    If<{ has_copy_dst(DST_USAGE) }>: True2,
  {
    self.encoder.copy_buffer_to_buffer(
      &source.buffer,
      source_offset,
      &destination.buffer,
      destination_offset,
      copy_size,
    );
  }
}

use wgpu::util::{BufferInitDescriptor, DeviceExt};

use super::*;

pub struct Buffer<const USAGE: wgpu::BufferUsages> {
  pub(super) buffer: wgpu::Buffer,
}

pub const INDEX: wgpu::BufferUsages = wgpu::BufferUsages::INDEX;
pub const VERTEX: wgpu::BufferUsages = wgpu::BufferUsages::VERTEX;
pub const COPY_DST: wgpu::BufferUsages = wgpu::BufferUsages::COPY_DST;
pub const COPY_SRC: wgpu::BufferUsages = wgpu::BufferUsages::COPY_SRC;

pub type IndexBuffer = Buffer<INDEX>;
pub type VertexBuffer = Buffer<VERTEX>;

impl<const USAGE: wgpu::BufferUsages> Buffer<USAGE> {
  pub fn new(device: &wgpu::Device, contents: &[u8]) -> Self {
    Self {
      buffer: device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents,
        usage: USAGE,
      }),
    }
  }
}

// example
// https://internals.rust-lang.org/t/const-generics-where-restrictions/12742/4
impl<const USAGE: wgpu::BufferUsages> Buffer<USAGE>
where
  If<{ can_map_read(USAGE) }>: True,
{
  // only_impl_when_buffer_has_read_ability
  pub fn read(&self) -> Self {
    todo!()
  }
}

pub const fn has_copy_src(usage: wgpu::BufferUsages) -> bool {
  usage.contains(COPY_SRC)
}

pub const fn has_copy_dst(usage: wgpu::BufferUsages) -> bool {
  usage.contains(COPY_DST)
}

// If Features::MAPPABLE_PRIMARY_BUFFERS isn’t enabled, the only other usage a buffer may have is COPY_DST.
pub const fn can_map_read(usage: wgpu::BufferUsages) -> bool {
  usage.bits() == COPY_DST.bits()
}

//  a | b, const fn in trait not support yet(except primitive type), so let's use a const fn and transmute!
pub const fn or(a: wgpu::BufferUsages, b: wgpu::BufferUsages) -> wgpu::BufferUsages {
  unsafe { std::mem::transmute(a.bits() | b.bits()) }
}

#[test]
#[allow(clippy::diverging_sub_expression)]
fn test() {
  let buffer_x: Buffer<{ COPY_DST }> = todo!();
  let buffer_a: Buffer<{ or(COPY_DST, INDEX) }> = todo!();
  let buffer_b: Buffer<{ or(or(COPY_DST, INDEX), COPY_DST) }> = todo!();
  let buffer_c: Buffer<{ INDEX }> = todo!();

  // compile
  buffer_x.read();

  // not compile
  // buffer_a.read();
  // buffer_b.read();
  // buffer_c.read();
}
