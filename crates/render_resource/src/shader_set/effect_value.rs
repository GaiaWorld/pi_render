use std::sync::Arc;

use crate::{unifrom_code::MaterialValueBindDesc, shader_bind::{ShaderBindEffectValue, TShaderBind}};


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderSetEffectValueAbout {
    pub set: u32,
    pub bind_effect: ShaderBindEffectValue,
}
impl ShaderSetEffectValueAbout {
    pub fn new(
        set: u32,
        effect: Arc<MaterialValueBindDesc>,
    ) -> Self {
        let mut bind = 0;
        
        let bind_effect = bind; bind += 1;
        let bind_effect = ShaderBindEffectValue::new(bind_effect, effect);
        // let bind_effect = if effect.has_value() {
        //     let bind_effect = bind; bind += 1;
        //     ShaderBindEffectValue::new(bind_effect, effect)
        // } else {
        //     ShaderBindEffectValue::new(u32::MAX, effect)
        // };

        Self {
            set,
            bind_effect
        }
    }
    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut entries = vec![];
        
        // 当未设置任何参数 会有 4 个 占位u32; 对应MaterialValueBindDesc中也有处理
        self.bind_effect.layout_entry(&mut entries);

        entries
    }
}
