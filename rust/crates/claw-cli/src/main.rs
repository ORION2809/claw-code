mod app;
mod args;
mod format;
mod init;
mod input;
mod render;
mod session_mgr;
mod tui;

fn main() {
    if let Err(error) = app::run() {
        eprintln!("{}", format::render_cli_error(&error.to_string()));
        std::process::exit(1);
    }
}
