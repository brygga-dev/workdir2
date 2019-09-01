use clap::Shell;

include!("src/cli.rs");

fn main() {
    let mut app = cli_app();
    app.gen_completions("wop", Shell::Bash, "/home/vagrant");
}
