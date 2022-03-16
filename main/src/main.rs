mod error;

pub use crate::error::Result;
use {
    ::error::te,
    std::{fs, io},
};

fn main() -> Result<()> {
    pretty_env_logger::init();

    const SAMPLE_PATH: &str = "test.dust";
    log::debug!("Loading {}", SAMPLE_PATH);
    let sample_text: String = te!(fs::read_to_string(SAMPLE_PATH));

    log::debug!("Parsing {}", SAMPLE_PATH);
    let module_ast = te!(parse::parse(&sample_text));
    log::info!("AST: {:#?}", module_ast);

    Ok(())
}
