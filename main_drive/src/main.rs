use app_error::{te, Result};

use parser_1::dust;

fn main() -> Result<()> {
    let mut sample_file = te!(std::fs::File::open("sample.dust"));
    let mut sample = String::new();
    te!(std::io::Read::read_to_string(&mut sample_file, &mut sample));

    let a = dust::ScriptParser::new().parse(&sample);
    let a = a.unwrap();
    eprintln!("{:#?}", a);
    Ok(())
}
