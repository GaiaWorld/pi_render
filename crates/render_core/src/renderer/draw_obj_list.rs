use std::time::Instant;

use crate::renderer::draw_obj::TempDrawInfoRecord;

use super::{draw_obj::{DrawObj, TBindGroups, TGeometry}, pipeline::TRenderPipeline};


pub struct DrawList<T: TRenderPipeline, B: TBindGroups, G: TGeometry> {
    pub list: Vec<DrawObj<T, B, G>>
}
impl<T: TRenderPipeline, B: TBindGroups, G: TGeometry> DrawList<T, B, G> {
    pub fn render<'a>(
        &self,
        commands: &'a mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        depth_stencil: Option<wgpu::RenderPassDepthStencilAttachment>,
    ) {
        let time = Instant::now();
        let draws = &self.list;

        let mut temp_vertex_record: TempDrawInfoRecord = TempDrawInfoRecord::default();

        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: true,
        };
        let mut color_attachments = vec![];
        color_attachments.push(
            Some(
                wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops,
                    view: target_view,
                }
            )
        );

        let mut renderpass = commands.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("RenderNode"),
                color_attachments: color_attachments.as_slice(),
                depth_stencil_attachment: depth_stencil,
            }
        );

        let mut draw_count = 0;
        draws.iter().for_each(|draw| {
            renderpass.set_pipeline(draw.pipeline.pipeline());
            draw.bindgroups.bindgroups().iter().for_each(|bindinfo| {
                renderpass.set_bind_group(bindinfo.set, &bindinfo.bind_group(), &bindinfo.offsets());
            });

            let mut vertex_range = 0..0;
            let mut instance_range = 0..1;
            draw.geo.vertices().iter().for_each(|item| {
                if temp_vertex_record.record_vertex_and_check_diff_with_last(item) {
                    renderpass.set_vertex_buffer(item.slot, item.slice());
                    vertex_range = item.value_range();
                }
            });

            draw.geo.instances().iter().for_each(|item| {
                if temp_vertex_record.record_vertex_and_check_diff_with_last(item) {
                    renderpass.set_vertex_buffer(item.slot, item.slice());
                    instance_range = item.value_range();
                }
            });

            match &draw.geo.indices() {
                Some(indices) => {
                    if temp_vertex_record.record_indices_and_check_diff_with_last(indices) {
                        renderpass.set_index_buffer(indices.slice(), indices.format);
                    }

                    renderpass.draw_indexed(indices.value_range(), 0 as i32, instance_range);
                },
                None => {
                    renderpass.draw(vertex_range, instance_range);
                },
            }
            draw_count += 1;
        });
        
        let time1 = Instant::now();
        log::info!("DrawList: {}, {:?}", draw_count, time1 - time);
    }
}