use toto::cli::Cli;

fn main() {
    let file_path = "./toto/".to_string();
    let _ = Cli::new(file_path).run();
}
