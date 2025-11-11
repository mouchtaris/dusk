use super::*;
use collection::Recollect;
use error::temg;

pub trait Cmd: Fn(Vec<String>) -> Result<()> {}
impl<S: Fn(Vec<String>) -> Result<()>> Cmd for S {}

pub fn xsi() -> impl Cmd {
    fn help() {
        eprintln!("compile      [- | IN_PATH.src] [- | OUT_PATH.obj]");
        eprintln!("decompile    [- | IN_PATH.obj] [- | OUT_PATH.txt]");
        eprintln!("dump         [- | IN_PATH.obj] [- | OUT_PATH.txt]");
        eprintln!("call         [- | IN_PATH.obj] FUNC_NAME [ARGS...]");
        eprintln!("ccall        [- | IN_PATH.src] FUNC_NAME [ARGS...]");
        eprintln!("crun         [- | IN_PATH.src] [ARGS...]");
        eprintln!("run          [- | IN_PATH.obj] [ARGS...]");
        eprintln!("link         [- | OUT_PATH.lib] [- | IN_PATH.obj...]     Generated lib files cannot be `run`.");
    }
    |mut args| {
        // Reverse args for easier traverse (.pop())
        args.reverse();
        // Pop exec-name
        let prog = args.pop();
        // Pop subcommand-name
        let subcommand = args.pop();
        // Repush prog-name as first arg
        prog.into_iter().for_each(|x| args.push(x));
        // Dispatch subcommand
        match subcommand.as_ref().map(String::as_str) {
            // ---- Legacy CLI commands ----
            // These are direct copies from original utilities that showed the way (xs-compile, xs-call, ..).
            // Instead of adapting *them*, we prehandle args here to give them as expected.
            // <3
            // *ALSO* they are still used by traditional utils, so they need to handle
            // actual CLI themselves as well.
            Some("compile") => te!(compile()(args.reversed())),
            Some("call") => te!(call()(args.reversed())),
            // ---- New CLI commands ----
            // These are new implementations for front-end.
            Some("decompile") => te!(decompile()(args)),
            Some("dump") => te!(dump()(args)),
            Some("ccall") => te!(compile_and_call()(args)),
            Some("crun") => te!(compile_and_run()(args)),
            Some("run") => te!(run()(args)),
            Some("link") => te!(link()(args)),
            Some("help") => help(),
            other => {
                help();
                temg!("Unknown command: {other:?}")
            }
        }
        Ok(())
    }
}

fn args<I: DoubleEndedIterator + ExactSizeIterator>(
    revargs: impl IntoIterator<IntoIter = I>,
    n: usize,
) -> impl ExactSizeIterator + DoubleEndedIterator<Item = I::Item> {
    revargs.into_iter().rev().skip(n)
}

pub fn link() -> impl Cmd {
    |revargs| {
        let args = |n| args(&revargs, n);

        let modules: Result<Vec<_>> = args(2).map(|path| load_compiler(&path)).collect();
        let modules = te!(modules);

        let module = compile::link::link_modules(modules);

        let mut output = te!(args_get_output(args(1)));
        te!(sd::ser(&mut output, &module));

        Ok(())
    }
}
pub fn run() -> impl Cmd {
    |revargs| {
        let args = |n| args(&revargs, n);

        let input = te!(args_get_input(args(1)));

        Ok(te!(run_vm_script(
            &mut te!(make_vm()),
            &te!(read_compiler(input)),
            args(2)
        )))
    }
}

pub fn compile_and_run() -> impl Cmd {
    |revargs| {
        let args = |n| args(&revargs, n);

        let input = args(1);

        Ok(te!(run_vm_script(
            &mut te!(make_vm()),
            &te!(compile_from_input(input)),
            args(2)
        )))
    }
}

pub fn compile_and_call() -> impl Cmd {
    |revargs| {
        let arg = |n| revargs.iter().rev().skip(n);
        let revrest = |n| revargs.iter().skip(n);

        Ok(te!(make_vm_call(
            &mut te!(make_vm()),
            &te!(compile_from_input(arg(1))),
            te!(arg(2).next(), "Missing func_addr"),
            revrest(3)
        )))
    }
}

pub fn dump() -> impl Cmd {
    |args| {
        let args = |n| args.iter().rev().skip(n);

        let _input = te!(args_get_input(args(1)));
        let mut input = args(1);
        let output = te!(args_get_output(args(2)));

        let icode = te!(compile_file(input.next().expect("input file path")));

        use show::Show;
        te!(icode.write_to(Ok(output)));

        Ok(())
    }
}

pub fn decompile() -> impl Cmd {
    |args| {
        let args = |n| args.iter().rev().skip(n);

        let input = te!(args_get_input(args(1)));
        let output = te!(args_get_output(args(2)));
        let icode = te!(read_compiler(input));

        use show::Show;
        te!(icode.write_to(Ok(output)));

        Ok(())
    }
}

// ----------------------------------------------------------------------------
// [!!] The origincal xs-call. Do not alter.
pub fn call() -> impl Cmd {
    |args| {
        let xs_call = || -> Result<()> {
            let mut args = te!(if_source_newer_than_target(args));

            args.reverse();
            args.pop(); // skip-prog-name

            let mut module_path = te!(args.pop(), "Missing module path");
            let func_addr = te!(args.pop(), "Missing function addr");

            if module_path == "-" {
                module_path.clear();
                module_path.push_str("/dev/stdin");
            }

            let compl = te!(load_compiler(&module_path), "ICode loading {}", module_path);

            let mut vm = te!(make_vm());
            te!(make_vm_call(&mut vm, &compl, &func_addr, args)
                .map_err(|err| err.with_comment(format!("Loading icode from {}", module_path))));
            Ok(())
        };
        return xs_call();

        fn last_mod(path: impl AsRef<std::path::Path>) -> Result<std::time::SystemTime> {
            Ok(te!(te!(std::fs::metadata(path)).modified()))
        }

        fn if_source_newer_than_target(mut args: Vec<String>) -> Result<Vec<String>> {
            let iter = args.iter().enumerate();

            const PATS: &[&str] = &["--if-source-newer=", "--than-target="];

            let now = std::time::SystemTime::now();
            let mut mods = [now, now];
            let mut idxs = [0, 0];

            for (i, arg) in iter {
                for (j, pat) in PATS.iter().copied().enumerate() {
                    if let Some(path) = arg.strip_prefix(pat) {
                        mods[j] = te!(last_mod(path));
                        idxs[j] = i;
                    }
                }
            }

            let [source_mod, target_mod] = mods;

            match () {
                _ if target_mod == source_mod => {}
                _ if target_mod > source_mod => {
                    std::process::exit(0);
                }
                _ => {
                    idxs.sort();
                    let [j, i] = idxs;
                    args.swap_remove(i);
                    args.swap_remove(j);
                }
            }

            Ok(args)
        }
    }
}

// ----------------------------------------------------------------------------
// [!!] The origincal xs-compile. Do not alter.
//
pub fn compile() -> impl Cmd {
    |args| {
        let input_path = args.get(1).map(String::as_str).unwrap_or("-");
        let output_path = args.get(2).map(String::as_str).unwrap_or("-");

        log::info!("Loading {}", input_path);
        let input_path = match input_path {
            "-" => "/dev/stdin",
            x => x,
        };
        let input_text: String = te!(
            fs::read_to_string(input_path),
            "Reading input file: {}",
            input_path
        );

        log::info!("Parsing {}", input_path);
        let module_ast = te!(parse::parse(&input_text));
        #[cfg(not(feature = "release"))]
        {
            use io::Write;
            te!(te!(fs::File::create("_.ast.txt")).write_fmt(format_args!("{:#?}", module_ast)));
        }

        log::info!("Compiling {}", input_path);
        let mut cmp = compile::Compiler::new();
        te!(cmp.init(&input_path));
        te!(cmp
            .compile(module_ast)
            .map_err(|err| err.with_comment(format!("Compiling {}", input_path))));
        #[cfg(feature = "debug")]
        {
            use show::Show;
            te!(cmp.write_to(fs::File::create("_.compiler.txt")));
        }

        let output_path = match output_path {
            "-" => "/dev/stdout",
            x => x,
        };
        log::info!("Writing to {}", output_path);
        let dst = te!(fs::File::create(&output_path), "Writing to {}", output_path);
        te!(sd::ser(dst, &cmp), "Serializing to {}", output_path);

        Ok(())
    }
}
