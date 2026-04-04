fn main() {
    if let Err(err) = dumap_cli::run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
