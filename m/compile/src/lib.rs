pub const VERSION: &str = "0.0.1";
use {
    ::show::Show,
    collection::Map,
    error::{ltrace, soft_todo, te, temg},
    std::io,
    vm::Instr as i,
};

error::Error! {
    Msg = &'static str
    Message = String
    Vm = vm::Error
}

mod compile;
mod compilers;
mod emit;
mod show;
pub mod symbol_info;
mod symbol_table;
pub use {
    crate::compile::{Compile, CompileEv},
    compilers::{Compilers, CompilersImpl as cmps},
    emit::EmitExt,
    symbol_info as sym,
    symbol_table::{SymbolTable, SymbolTableExt},
};

#[derive(Debug, Default)]
pub struct Compiler {
    pub icode: vm::ICode,
    sym_table: SymbolTable,
    /// For arbitrary use between compilations
    retval: usize,
}

impl Compiler {
    pub fn init(&mut self) {}

    pub fn compile<N>(self, node: &N) -> Result<Self>
    where
        Self: Compile<N>,
    {
        Compile::compile(self, node)
    }

    /// Add the given literal string to the string_table and
    /// return its id.
    fn add_string<S>(&mut self, s: S) -> Result<usize>
    where
        S: Into<String>,
    {
        let vm::StringInfo { id } = te!(vm::StringInfo::add(&mut self.icode, s));
        Ok(id)
    }

    /// Return the last instruction id
    pub fn instr_id(&self) -> usize {
        self.icode.instructions.len() - 1
    }

    /// ### Example
    ///
    ///     # use compile::Result;
    ///     # fn main() -> Result<()> {
    ///     use vm::Instr as i;
    ///     use compile::{Compiler, EmitExt};
    ///     use ::error::{te, temg};
    ///
    ///     let mut cmp = Compiler::default();
    ///     cmp.emit1(i::Allocate { size: 0 });
    ///     let instr_alloc = cmp.instr_id();
    ///
    ///     assert_eq!(
    ///         &cmp.icode.instructions[instr_alloc],
    ///         &i::Allocate { size: 0 }
    ///     );
    ///
    ///     te!(cmp.backpatch(instr_alloc, |i| match i {
    ///         i::Allocate { size } => Ok(*size = 4),
    ///         _ => temg!("not an allocate instr"),
    ///     }));
    ///
    ///     assert_eq!(
    ///         &cmp.icode.instructions[instr_alloc],
    ///         &i::Allocate { size: 4 }
    ///     );
    ///
    ///     # Ok(())
    ///     # }
    ///
    pub fn backpatch<B>(&mut self, instr_id: usize, block: B) -> Result<()>
    where
        B: FnOnce(&mut i) -> Result<()>,
    {
        let instr = te!(self.icode.instructions.get_mut(instr_id));
        block(instr)
    }
    pub fn backpatch_with(&mut self, instr_id: usize, val: usize) -> Result<()> {
        self.backpatch(instr_id, |i| {
            Ok(*match i {
                i::PushNat(v) => v,
                i::Allocate { size } => size,
                i::Jump { addr } => addr,
                other => temg!("Not a single usize value instruction, {:?}", other),
            } = val)
        })
    }

    /// Add the given string as a literal and emit to load it into `cmp.retval`
    fn compile_text<S>(self, text: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let mut cmp = self;

        let text = text.as_ref();
        let strid = te!(cmp.add_string(text));

        cmp.retval = te!(cmp
            .new_local_tmp(format_args!("literal-text-{}", strid))
            .fp_off());

        cmp.emit([i::PushStr(strid)]);

        Ok(cmp)
    }
}

impl SymbolTableExt for Compiler {}
impl EmitExt for Compiler {}
impl AsRef<SymbolTable> for Compiler {
    fn as_ref(&self) -> &SymbolTable {
        &self.sym_table
    }
}
impl AsMut<SymbolTable> for Compiler {
    fn as_mut(&mut self) -> &mut SymbolTable {
        &mut self.sym_table
    }
}
impl AsMut<Compiler> for Compiler {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
