fn main() {
    use main::{
        cli::{megafront, Cmd},
        run_main_app as run_app,
    };
    run_app(megafront().revargs())
}
