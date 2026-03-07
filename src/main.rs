use frr_prefix_gen::cli::Cli;
use frr_prefix_gen::run;

fn main() {
    let cli = Cli::parse_args();
    
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
