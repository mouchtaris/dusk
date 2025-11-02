use super::*;
use error::temg;

pub trait Cmd: Fn(Vec<String>) -> Result<()> {}
impl<S: Fn(Vec<String>) -> Result<()>> Cmd for S {}

pub fn xsi() -> impl Cmd {
    fn help() {
        eprintln!("compile [- | IN_PATH] [- | OUT_PATH]");
        eprintln!("call    [- | IN_PATH] FUNC_NAME [ARGS...]");
    }
    |args| {
        let subcommand = args[1].as_ref();
        match subcommand {
            "compile" => te!(compile()(args)),
            "call" => te!(call()(args)),
            other => {
                help();
                temg!("Unknown command: {other}")
            }
        }
        Ok(())
    }
}

pub fn call() -> impl Cmd {
    |args| {
        let xs_call = || -> Result<()> {
            let mut args = te!(if_source_newer_than_target(args));

            args.reverse();

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
