use pi_assets::asset::Handle;
use pi_map::vecmap::VecMap;
use wgpu::RenderPass;
use crate::rhi::shader::BindLayout;

use super::{
    asset::RenderRes, bind_group::BindGroup, buffer::Buffer, dyn_uniform_buffer::BufferGroup,
    pipeline::RenderPipeline, shader::Uniform,
};

/// 渲染对象
#[derive(Debug, Default)]
pub struct DrawState {
    // 一个 Pipeleine
    pub pipeline: Option<Handle<RenderRes<RenderPipeline>>>,

    // 一堆 UBO
    pub bind_groups: Groups,

    // 一堆 VB
    pub vbs: VecMap<(Handle<RenderRes<Buffer>>, u64)>,

    // IB 可有 可无
    pub ib: Option<(Handle<RenderRes<Buffer>>, u64, wgpu::IndexFormat)>,
}

#[derive(Debug, Default)]
pub struct Groups(VecMap<DrawGroup>);

impl Groups {
	pub fn set_uniform<T: Uniform>(&mut self, value: &T) {
		self.0[T::Binding::set() as usize].set_uniform(value);
    }

    #[inline]
    pub fn get_group(&self, group_id: u32) -> Option<&DrawGroup> {
        self.0.get(group_id as usize)
    }

    pub fn insert_group<T: Into<DrawGroup>>(&mut self, group_id: u32, value: T) {
        self.0.insert(group_id as usize, value.into());
    }

    #[inline]
    pub fn groups(&self) -> &VecMap<DrawGroup> {
        &self.0
    }
}

#[derive(Debug)]
pub enum DrawGroup {
    Offset(BufferGroup), // 某个buffer的部分 (在全局中的索引， buffer偏移量) 具有动态偏移
    Independ(Handle<RenderRes<BindGroup>>), // 无动态偏移
}

impl DrawGroup {
	pub fn set_uniform<T: Uniform>(&mut self, value: &T) {
        let _ = match self {
            DrawGroup::Offset(group) => group.set_uniform(value),
            DrawGroup::Independ(_group) => todo!(),
        };
    }

    pub fn draw<'w, 'a>(&'a self, rpass: &'w mut RenderPass<'a>, i: u32) {
        match self {
            DrawGroup::Offset(index) => {
                let group = index.get_group();
                rpass.set_bind_group(i as u32, group.bind_group, group.offsets);
            }
            DrawGroup::Independ(group) => rpass.set_bind_group(i as u32, group, &[]),
        };
    }
}

impl DrawState {
    pub fn draw<'w, 'a>(&'a self, rpass: &'w mut RenderPass<'a>) {
        if let (Some(p), Some(ib)) = (&self.pipeline, &self.ib) {
            rpass.set_pipeline(p);
            let mut i = 0;
            for r in self.bind_groups.groups().iter() {
                if let Some(group) = r {
                    group.draw(rpass, i as u32);
                }
                i += 1;
            }
            i = 0;
            for r in self.vbs.iter() {
                if let Some(vertex_buf) = r {
                    rpass.set_vertex_buffer(i as u32, (****vertex_buf.0).slice(..));
                }
                i += 1;
            }

            rpass.set_index_buffer((****ib.0).slice(..), ib.2);
            rpass.draw_indexed(0..ib.1 as u32, 0, 0..1);
        }
    }
}
