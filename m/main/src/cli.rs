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
        eprintln!("debug-run    [- | IN_PATH.obj] [ARGS...]");
        eprintln!("debug-call   [- | IN_PATH.obj] FUNC_NAME [ARGS...]");
        eprintln!("debug-ccall  [- | IN_PATH.src] FUNC_NAME [ARGS...]");
        eprintln!("");
        eprintln!("");
        eprintln!("mega ::");
        eprintln!("");
        eprintln!("     --compile!=false    :: compile inputs as text code");
        eprintln!("");
        eprintln!("  input : [ path/script , ... ]");
        eprintln!("");
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
            Some("debug-run") => te!(debug_run()(args)),
            Some("debug-call") => te!(debug_call()(args)),
            Some("debug-ccall") => te!(debug_compile_and_call()(args)),
            Some("mega") => te!(megafront()(args)),
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

pub fn megafront() -> impl Cmd {
    |revargs| {
        let args = |n| args(&revargs, n).map(String::as_str);

        #[derive(Default, Debug)]
        struct Opts<'a> {
            input_paths: Vec<&'a str>,
            input_scripts: Vec<&'a str>,
            /// 0 for next path, 1 for next script
            input_order: Vec<u8>,
            compile: bool,
            debug: bool,
            call: Option<&'a str>,
            dump_to: Option<&'a str>,
            list_func: Option<&'a str>,
            rest_args: Option<usize>,
        }
        let mut opts: Opts = Opts::new();
        impl<'a> Opts<'a> {
            pub fn new() -> Self {
                Self { ..<_>::default() }
            }
            pub fn rest_args(
                &self,
                revargs: &'a [String],
            ) -> impl ExactSizeIterator + DoubleEndedIterator<Item = &'a str> {
                let Self { rest_args, .. } = self;
                let start_at = rest_args.unwrap_or(revargs.len());
                self::args(revargs, start_at).map(String::as_str)
            }
        }

        fn as_path(s: &str) -> Option<ast::Path> {
            let tokens = lex::Lex::new(s);
            parse::dust::PathParser::new().parse(tokens).ok()
        }

        for (i, arg) in args(1).enumerate() {
            let i = i + 1;

            if opts.rest_args.is_some() {
                break;
            }

            match arg.split_once('=') {
                Some(("--compile", val)) if val != "false" => opts.compile = true,
                Some(("--debug", val)) if val != "false" => opts.debug = true,
                Some(("--call", val)) => opts.call = Some(val),
                Some(("--dump_to", val)) => opts.dump_to = Some(val),
                Some(("--list_func", val)) => opts.list_func = Some(val),
                Some((opt, _)) if opt.starts_with("--") => temg!("Unknown opt: {opt}"),
                None if arg == "--" => opts.rest_args = Some(i + 1),
                _ => {
                    if let Some(_) = as_path(arg) {
                        opts.input_paths.push(arg);
                        opts.input_order.push(0);
                    } else {
                        opts.input_scripts.push(arg);
                        opts.input_order.push(1);
                    }
                }
            }
        }

        let input_paths = &opts.input_paths[..];
        let input_scripts = &opts.input_scripts[..];

        error::ldebug!("megafront configured: {opts:?}");

        let compiler = &te!(match (&opts, input_paths, input_scripts,) {
            (_, [], []) => compile_from_input(["-"]),
            (Opts { compile: true, .. }, [input_path], []) => compile_file(input_path),
            #[cfg(feature = "has_code_tools")]
            (
                Opts {
                    compile: true,
                    input_order,
                    ..
                },
                inps @ [base_path, ..],
                scripts,
            ) => {
                type Inp = io::Result<Box<dyn io::Read>>;
                fn read_script(x: impl AsRef<str>) -> Inp {
                    Ok(Box::new(code_tools_util::io_ext::from_iter(
                        x.as_ref().as_bytes().to_owned().into_iter(),
                    )))
                }
                fn read_file(x: impl AsRef<str>) -> Inp {
                    Ok(Box::new(File::open(x.as_ref())?))
                }
                let mut inps0 = inps.into_iter().map(read_file);
                let mut inps1 = scripts.into_iter().map(read_script);
                let inps = input_order.into_iter().flat_map(|&x| match x {
                    0 => inps0.next(),
                    _ => inps1.next(),
                });

                let inps = te!(code_tools_util::stx::IterRead::new(inps));

                // use the first input path as base
                compile_input_with_base(inps, base_path)
            }
            #[cfg(feature = "has_code_tools")]
            (_, [], scripts) => {
                let inps = te!(code_tools_util::stx::IterRead::new(
                    scripts.into_iter().map(|x| Ok(x.as_bytes()))
                ));
                compile_input_with_base(inps, "./")
            }
            _ => todo!("{opts:?}"),
        });

        use std::fs::File;
        use std::io::stdout;

        fn try_dest(
            opt: &Option<&str>,
            func: impl FnOnce(&mut &mut dyn io::Write) -> io::Result<()>,
        ) -> Result<bool> {
            if let Some(dest) = opt {
                let mut out: &mut dyn io::Write = if dest.is_empty() {
                    &mut stdout()
                } else {
                    &mut te!(File::create(dest))
                };
                te!(func(&mut out));
                return Ok(true);
            }
            Ok(false)
        }

        if te!(try_dest(&opts.dump_to, |dest| compiler.write_out(dest)))
            || te!(try_dest(&opts.list_func, |dest| Ok({
                let mut sep = "";
                for func in list_func(&compiler) {
                    write!(dest, "{sep}{func}")?;
                    sep = "\x00";
                }
            })))
        {
            return Ok(());
        }

        let vm: &mut vm::Vm = &mut te!(make_vm());
        let cmp: &compile::Compiler = compiler;
        let revargs = opts.rest_args(&revargs[..]).rev();
        let debug: bool = opts.debug;

        Ok(if let Some(func_addr) = opts.call {
            te!(make_vm_call2(vm, cmp.to_owned(), func_addr, revargs, debug))
        } else {
            te!(run_vm_script(vm, cmp, revargs, debug))
        })
    }
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
            args(2),
            false
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
            args(2),
            false
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

pub fn debug_compile_and_call() -> impl Cmd {
    |revargs| {
        let args = |n| args(&revargs, n);

        te!(make_vm_call2(
            &mut te!(make_vm()),
            te!(compile_from_input(args(1))),
            te!(args(2).next(), "Missing func name"),
            args(3).rev(),
            true,
        ));
        Ok(())
    }
}

pub fn debug_run() -> impl Cmd {
    |revargs| {
        let args = |n| args(&revargs, n);

        let input = te!(args_get_input(args(1)));

        Ok(te!(run_vm_script(
            &mut te!(make_vm()),
            &te!(read_compiler(input)),
            args(2),
            true
        )))
    }
}

pub fn debug_call() -> impl Cmd {
    |revargs| {
        let args = |n| args(&revargs, n);

        te!(make_vm_call2(
            &mut te!(make_vm()),
            te!(read_compiler(te!(args_get_input(args(1))))),
            te!(args(2).next(), "Missing func name"),
            args(3).rev(),
            true,
        ));
        Ok(())
    }
}

pub fn dump() -> impl Cmd {
    |args| {
        let args = |n| args.iter().rev().skip(n);

        let _input = te!(args_get_input(args(1)));
        let input = args(1);
        let output = te!(args_get_output(args(2)));

        let icode = te!(compile_from_input(input));

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
                _ if target_mod > source_mod => {
                    std::process::exit(0);
                }
                _ => {
                    let [j, i] = idxs;
                    args.remove(j);
                    args.remove(i);
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
