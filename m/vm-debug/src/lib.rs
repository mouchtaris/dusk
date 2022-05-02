use {
    compile::Compiler,
    error::te,
    std::{fmt, io},
    vm::{value, Result, Value, Vm, DEBUG_STACK_SIZE},
};

pub fn write_to<O>(vm: &Vm, cmp: &Compiler, o: io::Result<O>) -> Result<()>
where
    O: io::Write,
{
    let mut o = te!(o);

    let mut strbuf = String::new();
    use fmt::Write;

    use ::error::te_writeln as w;
    //w!(o, "=== BIN_PATH ===")?;
    //for path in &vm.bin_path {
    //    w!(o, "> {}", path)?;
    //}
    w!(o, "=== STRING TABLE ===");
    let mut i = 0;
    for string in vm.string_table() {
        w!(o, "[{:4}] {:?}", i, string);
        i += 1;
    }
    let mut fp = vm.frame_ptr();
    let mut sp = vm.stack_ptr();
    let len = vm.stack_len();
    let mut count = 0;
    w!(o, "=== STACK ===");
    w!(o, "fp({fp}) sp({sp}) len({l})", l = len, sp = sp, fp = fp);
    for i in 0..len {
        let i = len - 1 - i;

        let too_far = sp + 3;
        if i > too_far {
            continue;
        }
        if i == too_far {
            w!(o, "  ... (too far)");
        }

        count += 1;
        if count > DEBUG_STACK_SIZE {
            break;
        }

        let stack = vm.stack();
        let pref = if fp < 7 {
            "(sys)"
        } else {
            let nargs: usize = *te!(stack[fp - 3].try_ref());
            let n_inp_redir: usize = *te!(stack[fp - 3 - nargs - 3].try_ref());
            let nenvs: usize = *te!(stack[fp - 3 - nargs - 3 - n_inp_redir - 1].try_ref());
            match i {
                i if fp == i => "fp",
                i if fp - 1 == i => "ret instr",
                i if fp - 2 == i => "ret frame",
                i if fp - 3 == i => "nargs",
                i if fp - 3 - nargs <= i && fp - 3 > i => "arg",
                i if fp - 3 - nargs - 1 == i => "cwd",
                i if fp - 3 - nargs - 2 == i => "target",
                i if fp - 3 - nargs - 3 == i => "n inp redr",
                i if fp - 3 - nargs - 3 - n_inp_redir <= i && fp - 3 - nargs - 3 > i => "inp redr",
                i if fp - 3 - nargs - 3 - n_inp_redir - 1 == i => "nenvs",
                i if fp - 3 - nargs - 3 - n_inp_redir - 1 - 2 * nenvs <= i
                    && fp - 3 - nargs - 3 - n_inp_redir - 1 > i =>
                {
                    "env set"
                }
                i if fp - 3 - nargs - 3 - n_inp_redir - 1 - 2 * nenvs - 1 == i => {
                    sp = *te!(stack[fp - 1].try_ref());
                    fp = *te!(stack[fp - 2].try_ref());
                    w!(o, "--- frame {} ---", fp);
                    "retval"
                }
                i if sp == i => "sp",
                _ => "",
            }
        };
        let cell = &stack[i];
        strbuf.clear();
        te!(write!(strbuf, "{:?}", cell));
        let explain_start = strbuf.len();
        match cell {
            &Value::LitString(value::LitString(strid)) => {
                te!(write!(strbuf, "{:?}", te!(vm.get_string_id(strid))))
            }
            &Value::Job(value::Job(jobid)) => {
                te!(write!(strbuf, "{:?}", te!(vm.get_job(jobid))))
            }
            &Value::Natural(val) => te!(write!(strbuf, "{}", val)),
            Value::FuncAddr(value::FuncAddr(faddr)) => {
                let name = compile::find_func_name(cmp, faddr).unwrap_or("<n/a>");
                te!(write!(strbuf, "def {}", name))
            }
            _ => (),
        };
        let cell_str = &strbuf[..explain_start];
        let explain = &strbuf[explain_start..];
        w!(o, "{:10} [{:4}] {:29} | {}", pref, i, cell_str, explain);
    }
    if count > DEBUG_STACK_SIZE {
        w!(o, " ... (stack elided)");
    }
    w!(o, "=== STATE ===");
    w!(o, "- frame pointer    : {}", vm.frame_ptr());
    w!(o, "- stack pointer    : {}", vm.stack_ptr());
    w!(o, "- instr pointer    : {}", vm.instr_addr());
    Ok(())
}
