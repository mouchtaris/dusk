fn main() {
    use main::{
        cli::{megafront, Cmd},
        run_app,
    };
    run_app(megafront().revargs())
}
