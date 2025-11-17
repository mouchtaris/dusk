use std::borrow::Borrow;

use super::*;
use error::temg;

pub trait Cmd: Fn(Vec<String>) -> Result<()> {
    fn revargs(&self) -> impl Cmd {
        use collection::Recollect;
        move |args| self(args.reversed())
    }
}
impl<S: Fn(Vec<String>) -> Result<()>> Cmd for S {}

fn xsi_help() {
    eprintln!(
        r#"
compile      [- | IN_PATH.src] [- | OUT_PATH.obj]
decompile    [- | IN_PATH.obj] [- | OUT_PATH.txt]
dump         [- | IN_PATH.obj] [- | OUT_PATH.txt]
call         [- | IN_PATH.obj] FUNC_NAME [ARGS...]
ccall        [- | IN_PATH.src] FUNC_NAME [ARGS...]
crun         [- | IN_PATH.src] [ARGS...]
run          [- | IN_PATH.obj] [ARGS...]
link         [- | OUT_PATH.lib] [- | IN_PATH.obj...]     Generated lib files cannot be `run`.
debug-run    [- | IN_PATH.obj] [ARGS...]
debug-call   [- | IN_PATH.obj] FUNC_NAME [ARGS...]
debug-ccall  [- | IN_PATH.src] FUNC_NAME [ARGS...]


mega ::

  --compile!=false              -c  :: compile inputs as text code
  --debug!=false                -d  :: enable debugger when running
  --debug-do-system-main!=false -ds :: do not skip system main init when debugging
  --call=func_addr              -l  :: call function name instead of running script body
  --dump_to=dest_path               :: dump compiled object to dest_path (- stdout)
  --list_funcs_to=dest_path         :: write null-separated-list of global functions to dest_path
  --also_run!=false             -r  :: --dump* and --list* options will not run unless this

  input : [ path/script , ... ]
  -- [ script-args ... ]
"#
    )
}
pub fn xsi() -> impl Cmd {
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
            Some("compile") => te!(compile().revargs()(args)),
            Some("call") => te!(call().revargs()(args)),
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
            Some("help") => xsi_help(),
            other => {
                xsi_help();
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
            also_run: bool,
            debug: bool,
            debug_do_system_main: bool,
            call: Option<&'a str>,
            dump_to: Option<&'a str>,
            dump_text_to: Option<&'a str>,
            base_path: Option<&'a str>,
            list_funcs_to: Option<&'a str>,
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
            pub fn base_path(&'a self, or: impl FnOnce() -> &'a str) -> &'a str {
                self.base_path.unwrap_or_else(or)
            }
            pub fn set(&mut self, s: Set, i: usize, x: &'a str) {
                s(self, i, x)
            }
        }

        fn as_path(s: &str) -> Option<ast::Path> {
            let tokens = lex::Lex::new(s);
            parse::dust::PathParser::new().parse(tokens).ok()
        }

        type Set = for<'r> fn(&mut Opts<'r>, usize, &'r str);
        let mut setting: Option<Set> = None;
        macro_rules! set {
            ($name:ident, set_func, $func:expr) => {
                let $name: Set = $func;
            };
            ($name:ident, func) => {
                |Opts { $name, .. }: &mut Opts, _: usize, x: &str| *$name = Some(x)
            };
            ($name:ident) => {
                set!($name, set_func, set!($name, func))
            };
            ($name:ident, map, $map:expr) => {
                set!(
                    $name,
                    set_func,
                    (|opts, i, x| {
                        set!($name);
                        if x.is_empty() {
                            panic!(concat!(stringify!(name), " canot be empty"))
                        }
                        $name(opts, i, x)
                    })
                )
            };
        }

        for (i, arg) in args(1).enumerate() {
            let i = i + 1;

            if opts.rest_args.is_some() {
                break;
            }

            if let Some(lens) = &mut setting {
                lens(&mut opts, i, arg);
                setting = None;
                continue;
            }
            set!(call);
            set!(base_path, map, non_empty);
            let mut set = |s: Set| {
                setting = Some(s);
            };

            match arg.split_once('=') {
                Some(("--compile", val)) if val != "false" => opts.compile = true,
                Some(("--also_run", val)) if val != "false" => opts.also_run = true,
                Some(("--debug", val)) if val != "false" => opts.debug = true,
                Some(("--debug-do-system-main", val)) if val != "false" => {
                    opts.debug_do_system_main = true
                }
                Some(("--call", val)) => opts.set(call, i, val),
                Some(("--dump_to", val)) => opts.dump_to = Some(val),
                Some(("--list_funcs_to", val)) => opts.list_funcs_to = Some(val),
                Some(("--dump_text_to", val)) => opts.dump_text_to = Some(val),
                Some(("--base_path", val)) => opts.set(base_path, i, val),
                Some((opt, _)) if opt.starts_with("--") => {
                    xsi_help();
                    temg!("Unknown opt: {opt}")
                }
                None if arg == "--" => opts.rest_args = Some(i + 1),
                _ => match arg {
                    "-c" => opts.compile = true,
                    "-r" => opts.also_run = true,
                    "-d" => opts.debug = true,
                    "-ds" => opts.debug_do_system_main = true,
                    "-l" => set(call),
                    "-b" => set(base_path),
                    _ => {
                        if let Some(_) = as_path(arg) {
                            opts.input_paths.push(arg);
                            opts.input_order.push(0);
                        } else {
                            opts.input_scripts.push(arg);
                            opts.input_order.push(1);
                        }
                    }
                },
            }
        }

        let input_paths = &opts.input_paths[..];
        let input_scripts = &opts.input_scripts[..];

        error::ldebug!("megafront configured:\n{revargs:#?}\n{opts:#?}");

        use std::fs::File;
        use std::io::stdin;
        use std::io::stdout;
        use std::io::Cursor;

        type Inp = io::Result<Box<dyn io::Read>>;
        fn read_script(x: impl AsRef<str>) -> Inp {
            let x = x.as_ref();
            Ok(if x == "-" {
                Box::new(stdin())
            } else {
                Box::new(Cursor::new(x.as_bytes().to_owned()))
            })
        }
        fn read_file(x: impl AsRef<str>) -> Inp {
            Ok(Box::new(File::open(x.as_ref())?))
        }

        // ---- Actual code action begins here ----

        let compiler = &te!(match (&opts, input_paths, input_scripts,) {
            (Opts { compile: true, .. }, [], []) => compile_from_input(["-"]),
            (Opts { compile: true, .. }, [input_path], []) => compile_file(input_path),
            #[cfg(feature = "has_code_tools")]
            (
                Opts {
                    compile: true,
                    input_order,
                    base_path,
                    ..
                },
                files,
                scripts,
            ) => {
                let mut inps0 = files.into_iter().map(read_file);
                let mut inps1 = scripts.into_iter().map(read_script);
                let inps = input_order.into_iter().flat_map(|&x| match x {
                    0 => inps0.next(),
                    _ => inps1.next(),
                });

                let inps = te!(code_tools_util::stx::IterRead::new(inps));

                // use the first input path as base
                let base_path = base_path
                    .or_else(|| files.first().map(|&s| s))
                    .unwrap_or("./");
                compile_input_with_base(inps, base_path)
            }
            #[cfg(feature = "has_code_tools")]
            (opts, [], scripts @ [_, ..]) => compile_input_with_base(
                te!(code_tools_util::stx::IterRead::new(
                    scripts.into_iter().map(read_script)
                )),
                opts.base_path(|| "./"),
            ),
            #[cfg(feature = "has_code_tools")]
            (_, paths @ [_, ..], []) => read_compiler(te!(code_tools_util::stx::IterRead::new(
                paths.into_iter().map(read_file)
            ))),
            (_, [], []) => read_compiler(stdin()),
            _ => todo!("{opts:?}"),
        });

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

        let dump_to = te!(try_dest(&opts.dump_to, |dest| compiler.write_out(dest)));
        let dump_text_to = te!(try_dest(&opts.dump_text_to, |dest| show::Show::write_to(
            compiler,
            Ok(dest)
        )));
        let list_funcs_to = te!(try_dest(&opts.list_funcs_to, |dest| Ok({
            let mut sep = "";
            for func in list_func(&compiler) {
                write!(dest, "{sep}{func}")?;
                sep = "\x00";
            }
        })));

        if !opts.also_run && (dump_to || dump_text_to || list_funcs_to) {
            return Ok(());
        }

        let vm: &mut vm::Vm = &mut te!(make_vm());
        let cmp: &compile::Compiler = compiler;
        let revargs = opts.rest_args(&revargs[..]).rev();

        Ok(if let Some(func_addr) = opts.call {
            te!(make_vm_call2(
                vm,
                cmp.to_owned(),
                func_addr,
                revargs,
                opts.debug
            ))
        } else {
            te!(run_vm_script(
                vm,
                cmp,
                revargs,
                (opts.debug, opts.debug_do_system_main)
            ))
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
            (false, false)
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
            (false, false)
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
            (true, false)
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
            let mut idxs = [None, None];

            for (i, arg) in iter {
                for (j, pat) in PATS.iter().copied().enumerate() {
                    if let Some(path) = arg.strip_prefix(pat) {
                        mods[j] = te!(last_mod(path));
                        idxs[j] = Some(i);
                    }
                }
            }

            let [source_mod, target_mod] = mods;
            if target_mod > source_mod {
                std::process::exit(0);
            }

            idxs.into_iter().flatten().for_each(|i| {
                args.remove(i);
            });

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
