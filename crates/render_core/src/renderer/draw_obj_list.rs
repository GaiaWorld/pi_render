use std::sync::Arc;

use crate::renderer::draw_obj::TempDrawInfoRecord;

use super::draw_obj::DrawObj;

#[derive(Default)]
pub struct DrawList {
    pub list: Vec<Arc<DrawObj>>,
    /// x, y, w, h, min_depth, max_depth
    /// 数值 0. ~ 1., 在设置视口时 应用到具体的视口上
    pub viewport: (f32, f32, f32, f32, f32, f32),
}
impl DrawList {
    pub fn render<'a, T: AsRef<DrawObj>>(
        draws: &'a [T],
        renderpass: & mut wgpu::RenderPass<'a>,
    ) {
        // let time = pi_time::Instant::now();

        let mut temp_vertex_record: TempDrawInfoRecord = TempDrawInfoRecord::default();
        let mut pipelinekey = 0;
        let mut draw_count: u64 = 0;
        draws.iter().for_each(|draw| {
            let draw = draw.as_ref();
            let vertex_range = draw.vertex.clone();
            let instance_range = draw.instances.clone();

            if let Some(pipeline) = &draw.pipeline {
                let key = pipeline.key().clone();
                if key != pipelinekey {
                    pipelinekey = key;
                    renderpass.set_pipeline(pipeline);
                }

                // draw.bindgroups.set(renderpass);
                for (item, idx) in draw.bindgroups.groups().iter() {
                    if *idx > 1 || temp_vertex_record.record_bindgroup_and_check_diff_with_last(*idx as usize, Some(item)) {
                        item.set(renderpass, *idx);
                    }
                }

                draw.vertices.iter().for_each(|(item, _)| {
                    // log::info!("vertex_range {:?}", item.buffer_range.clone());
					// log::info!("vertex_range {:?}", item.value_range().clone());
					if temp_vertex_record.record_vertex_and_check_diff_with_last(item) {
						renderpass.set_vertex_buffer(item.slot, item.slice());
					}
                });
    
                // log::info!("vertex_range {:?}", vertex_range.clone());

                match &draw.indices {
                    Some(indices) => {
                        if temp_vertex_record.record_indices_and_check_diff_with_last(indices) {
                            // log::warn!("Buffer  {:?}", indices.buffer.buffer());
                            renderpass.set_index_buffer(indices.slice(), indices.format);
                        }
                        // log::warn!("indices {:?}", indices.value_range());
                        renderpass.draw_indexed(indices.value_range(), 0 as i32, instance_range);
                    },
                    None => {
                        renderpass.draw(vertex_range, instance_range);
                    },
                }
                draw_count += 1;
            }
        });
        
        // let time1 = pi_time::Instant::now();
        // log::info!("DrawList: {}, {:?}", draw_count, time1 - time);
    }
}