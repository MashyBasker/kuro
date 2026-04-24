use std::{env, path::Path};

use crate::{cli::Commands, types::Project};

mod cli;
mod core;
mod types;
mod utils;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        utils::show_help();
        return;
    }

    if args[0] == "-h" || args[0] == "--help" {
        utils::show_help();
        return;
    }

    let cmd_parse_output = cli::parse_command(args);

    let result = match cmd_parse_output {
        Some(Commands::Build(path)) => {
            let project = Project::new(Path::new(&path).to_path_buf());
            core::build_site(&project)
        }
        Some(Commands::Init(path)) => core::create_site_directory(Path::new(&path)).map(|_| ()),
        Some(Commands::Serve { path, watch }) => {
            let project = Project::new(Path::new(&path).to_path_buf());
            core::serve(&project, watch)
        }
        Some(Commands::New { name, post }) => {
            let project = Project::new(Path::new(".").to_path_buf());
            core::create_new_file(&project, &name, post)
        }
        None => {
            utils::show_help();
            return;
        }
    };

    if let Err(e) = result {
        eprintln!("\n  ✗ Error: {e}\n");
        std::process::exit(1);
    }
}
