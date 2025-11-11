use {
    super::{env, fs, io, sd, Result},
    compile::{ScopeRef, SymbolTableExt},
    error::{ldebug, te, temg},
};

pub fn load_icode(input_path: &str) -> Result<vm::ICode> {
    Ok(te!(load_compiler(input_path)).icode)
}

pub fn load_compiler(input_path: &str) -> Result<compile::Compiler> {
    ldebug!("Loading {}", input_path);
    Ok(te!(read_compiler(&mut te!(fs::File::open(input_path)))))
}

pub fn read_compiler<R: io::Read>(mut input: R) -> Result<compile::Compiler> {
    let inp: Vec<u8> = {
        let mut inp: Vec<u8> = vec![];
        te!(io::Read::read_to_end(&mut input, &mut inp));

        if !inp.is_empty() && inp[0] == b'#' {
            let len = inp.len();
            let hashbang_end = inp.iter().cloned().take_while(|&b| b != b'\n').count();
            let hashbang_seg = te!(std::str::from_utf8(&inp[0..=hashbang_end]));
            ldebug!(
                "truncate {} {}:{:?} -> {:x}",
                len,
                hashbang_seg.len(),
                hashbang_seg,
                inp[hashbang_end + 1]
            );
            inp.copy_within(hashbang_end + 1.., 0);
            inp.truncate(len - hashbang_end - 1);
        }

        inp
    };

    ldebug!("loading {} -> {:x}", inp.len(), &inp[0]);
    let cmp = te!(sd::deser(inp.as_slice()));
    Ok(cmp)
}

/// Compile from the first string in the input-iter.
///
/// If the string is "-" read from stdin, otherwise delegate to [compile_file].
pub fn compile_from_input<I: IntoIterator>(input: I) -> Result<compile::Compiler>
where
    I::Item: AsRef<str>,
{
    let input = input.into_iter().next();
    let input = input.as_ref().map(<_>::as_ref).unwrap_or("-");

    Ok(if input == "-" {
        let cwd = te!(env::current_dir());
        let inp = io::stdin();
        te!(compile_input_with_base(
            inp,
            cwd.to_str().unwrap_or("/dev/stdin")
        ))
    } else {
        te!(compile_file(input))
    })
}

/// Read file content and delegate to [compile_input_with_base].
pub fn compile_file(input_path: &str) -> Result<compile::Compiler> {
    Ok(te!(compile_input_with_base(
        te!(fs::File::open(input_path)),
        input_path
    )))
}

pub fn compile_input_with_base(input: impl io::Read, base_path: &str) -> Result<compile::Compiler> {
    let input_text = te!(io::read_to_string(input));
    let module_ast =
        te!(parse::parse(&input_text)
            .map_err(|err| err.with_comment(format!("Parsing: {base_path}"))));
    let mut compiler = compile::Compiler::new();
    te!(compiler.init(base_path));
    te!(compiler
        .compile(module_ast)
        .map_err(|err| err.with_comment(format!("Compiling: {base_path}"))));
    Ok(compiler)
}

pub fn make_vm() -> Result<vm::Vm> {
    let mut vm = vm::Vm::default();
    vm.reset();
    te!(vm.init_bin_path_from_path_env());
    Ok(vm)
}

pub fn run_vm_script<T: ExactSizeIterator>(
    vm: &mut vm::Vm,
    compile::Compiler { icode, .. }: &compile::Compiler,
    args: impl IntoIterator<IntoIter = T, Item = T::Item>,
) -> Result<()>
where
    T::Item: Into<String>,
{
    let mut vm = te!(make_vm());
    te!(vm.init(args));
    te!(vm.eval_icode(icode));
    Ok(())
}

pub fn make_vm_call<Args: IntoIterator>(
    vm: &mut vm::Vm,
    cmp: &compile::Compiler,
    func_addr: &str,
    revargs: Args,
) -> Result<()>
where
    Args::IntoIter: ExactSizeIterator,
    Args::Item: Into<String>,
{
    let sinfo = cmp.lookup_by_name_everywhere(func_addr).ok();
    Ok(match sinfo {
        None => temg!("Function not found: {}", func_addr),
        Some(sym_id) => {
            let sinfo = sym_id.sym_info();
            let addr = te!(sinfo.as_addr_ref()).addr;
            te!(vm.init(revargs));
            vm.jump(addr);
            te!(vm.eval_icode(&cmp.icode));
            te!(vm::Instr::CleanUp(0).operate_on(vm));
        }
    })
}

pub fn script_call_getret(
    cmp: &compile::Compiler,
    func_addr: &str,
    args: Vec<String>,
) -> Result<Vec<u8>> {
    let vm = &mut te!(make_vm());
    let sinfo = cmp.lookup_by_name_everywhere(func_addr).ok();
    return Ok(match sinfo {
        None => temg!("Function not found: {}", func_addr),
        Some(sym_id) => {
            let sinfo = sym_id.sym_info();
            let addr = te!(sinfo.as_addr_ref()).addr;
            te!(vm.init(args));
            vm.jump(addr);
            te!(vm.eval_icode(&cmp.icode));
            //te!(vm::Instr::CleanUp(0).operate_on(vm));
            te!(take_buf(vm))
        }
    });
    fn take_buf(vm: &mut vm::Vm) -> Result<Vec<u8>> {
        Ok(match te!(vm.stack_get_val(0)) {
            vm::Value::Null(_) => todo!(),
            vm::Value::LitString(_) => todo!(),
            vm::Value::DynString(_) => todo!(),
            vm::Value::Natural(_) => todo!(),
            vm::Value::Array(_) => todo!(),
            &vm::Value::Job(vm::value::Job(id)) => match te!(vm.get_job_mut(id)) {
                job::Job::Null(_) => todo!(),
                job @ job::Job::Spec(_) => {
                    te!(job.make_buffer());
                    te!(take_buf(vm))
                }
                job::Job::System(_) => todo!(),
                job::Job::Buffer(buf) => std::mem::take(buf).take_bytes(),
            },
            vm::Value::FuncAddr(_) => todo!(),
            vm::Value::SysCallId(_) => todo!(),
            vm::Value::ArrayView(_) => todo!(),
        })
    }
}

pub fn list_func(cmp: &compile::Compiler) -> impl Iterator<Item = &str> {
    use collection::Recollect;
    cmp.global_scope_opt()
        .iter()
        .flat_map(|scope| scope.symbols())
        .filter_map(|(name, sym_id)| {
            let info = sym_id.sym_info();
            if info.scope_id == 1 && !name.starts_with('_') && info.as_addr_ref().is_ok() {
                return Some(name);
            }
            None
        })
        .sorted()
        .into_iter()
}

pub fn args_get_input<A>(args: A) -> Result<Box<dyn io::Read>>
where
    A: IntoIterator,
    A::Item: AsRef<str>,
{
    Ok(match args.into_iter().next().as_ref().map(<_>::as_ref) {
        Some("-") | None => Box::new(io::stdin()),
        Some(path) => te!(fs::File::open(path).map(Box::new), "input path: {}", path),
    })
}

pub fn args_get_output<A>(args: A) -> Result<Box<dyn io::Write>>
where
    A: IntoIterator,
    A::Item: AsRef<str>,
{
    Ok(match args.into_iter().next().as_ref().map(<_>::as_ref) {
        Some("-") | None => Box::new(io::stdout()),
        Some(path) => te!(fs::File::open(path).map(Box::new), "output path: {}", path),
    })
}
