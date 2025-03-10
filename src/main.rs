use toto::cli::Cli;

fn main() {
    let file_path = "/Users/tydelargy/.toto".to_string();
    let res = Cli::new(file_path).run();
    print!("{:?}", res);
}
