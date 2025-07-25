#[derive(Debug, Clone)]
pub struct Frame {
    pub name: String,
    pub stmts: std::sync::Arc<[viviscript_core::ast::Stmt]>,
    pub pc: usize,
}

impl Frame {
    pub fn new(name: String,stmts: impl Into<std::sync::Arc<[viviscript_core::ast::Stmt]>>, pc: usize) -> Self {
        Self {stmts: stmts.into(), pc, name}
    }
    
    pub fn current(&self) -> Option<&viviscript_core::ast::Stmt> {
        self.stmts.get(self.pc)
    }
    
    pub fn advance(&mut self) {
        self.pc += 1;
    }
}