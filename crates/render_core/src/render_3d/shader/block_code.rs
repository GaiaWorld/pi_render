use pi_atom::Atom;

pub trait TToBlockCodeAtom {
    fn to_block_code(&self) -> BlockCodeAtom;
}

impl TToBlockCodeAtom for crate::rhi::shader::BlockCodeAtom {
    fn to_block_code(&self) -> BlockCodeAtom {
        let mut code_define = String::from("");
        for c in self.define.iter() {
            c.code.to_code(&mut code_define);
        }
        
        let mut code_running = String::from("");
        for c in self.running.iter() {
            c.code.to_code(&mut code_running);
        }
        
        BlockCodeAtom {
            define: Atom::from(code_define),
            running: Atom::from(code_running),
        }
    }
}

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

/// 代码片段
#[derive(Debug, Clone)]
pub struct CodeSnippet(pub String);
impl CodeSnippet {
    pub fn size(&self) -> usize {
        self.0.as_bytes().len()
    }
}