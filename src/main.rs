use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Create,
    Merge {
        #[arg(short = 'o', long = "output", default_value = None)]
        output: Option<std::path::PathBuf>,
    },
}

fn main() {
    let args = Cli::parse();
    println!("{:?}", args.cmd);

    match args.cmd {
        Commands::Create => {

        },
        Commands::Merge { output } => {

        },
    }
}
