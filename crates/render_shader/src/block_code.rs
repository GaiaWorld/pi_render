use pi_atom::Atom;

/// 代码片段
#[derive(Debug, Clone)]
pub struct BlockCodeAtom {
    /// 声明代码
    pub define: Atom,
    /// 运行代码
    pub running: Atom,
}
impl BlockCodeAtom {
    pub fn size(&self) -> usize {
        self.define.as_bytes().len() + self.running.as_bytes().len()
    }
    pub fn to_block_code(&self) -> BlockCode {
        BlockCode {
            define: String::from(self.define.as_str()),
            running: String::from(self.running.as_str()),
        }
    }
}

/// 代码片段
#[derive(Debug, Clone)]
pub struct BlockCode {
    /// 声明代码
    pub define: String,
    /// 运行代码
    pub running: String,
}
impl BlockCode {
    pub fn size(&self) -> usize {
        self.define.as_bytes().len() + self.running.as_bytes().len()
    }
}