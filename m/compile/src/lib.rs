pub const VERSION: &str = "0.0.1";
use {
    ::show::Show,
    collection::{Deq, Map},
    error::{te, temg},
    std::{
        borrow::{Borrow, BorrowMut},
        io, mem, num,
    },
    vm::Instr as i,
};

error::Error! {
    Msg = &'static str
    Message = String
    Vm = vm::Error
    ParseDust = parse::Error
    ParseInt = num::ParseIntError
    Io = io::Error
    Sd2 = buf::sd2::Error
}

mod compile;
mod compile_util;
mod compilers;
mod emit;
pub mod facade;
mod file_path;
mod include;
mod sd;
mod show;
pub mod symbol_info;
mod symbol_table;
pub use {
    crate::compile::{Compile, CompileEv},
    compile_util::CompileUtil,
    compilers::{Compilers, CompilersImpl as cmps},
    emit::EmitExt,
    file_path::FilePathExt,
    include::IncludeExt,
    symbol_info as sym,
    symbol_table::{scopes, SymInfo, SymbolTable, SymbolTableExt},
};

#[derive(Default, Debug)]
pub struct Compiler {
    pub icode: vm::ICode,
    pub sym_table: SymbolTable,
    pub(in crate) current_file_path: Vec<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            icode: <_>::default(),
            sym_table: <_>::default(),
            current_file_path: <_>::default(),
        }
    }

    pub fn write_out<O: io::Write>(&self, out: &mut O) -> io::Result<()> {
        buf::sd2::WriteOut::write_out(self, out)
    }

    pub fn read_in<I: io::Read>(inp: &mut I) -> Result<Self> {
        Ok(te!(buf::sd2::ReadIn::read_in(inp)))
    }

    pub fn init(&mut self, file_path: &str) -> Result<()> {
        let cmp = self;

        cmp.enter_scope();
        cmp.push_file_path(file_path);

        Ok(())
    }

    pub fn compile<N>(&mut self, node: N) -> Result<<Self as Compile<N>>::RetVal>
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
    ///     let mut cmp = Compiler::new();
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
    fn compile_text<S>(&mut self, text: S) -> Result<SymInfo>
    where
        S: AsRef<str>,
    {
        let cmp = self;

        let text = text.as_ref();
        let strid = te!(cmp.add_string(text));

        Ok(SymInfo::lit_string(strid))
    }

    fn compile_natural<S>(&mut self, text: S) -> Result<SymInfo>
    where
        S: AsRef<str>,
    {
        let _cmp = self;

        let text = text.as_ref();
        let nat = te!(text.parse::<usize>());

        Ok(SymInfo::lit_natural(nat))
    }

    fn compile_funcaddr<S>(&mut self, text: S) -> Result<SymInfo>
    where
        S: AsRef<str>,
    {
        let cmp = self;

        let text = text.as_ref();
        let name = text;

        match te!(cmp.lookup(name)) {
            sinfo @ SymInfo {
                typ: sym::Typ::Address(_),
                ..
            } => Ok(sinfo.to_owned()),
            other => temg!("Not a function address {}: {:?}", name, other),
        }
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
