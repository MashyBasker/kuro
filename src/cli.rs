#[derive(Debug)]
pub enum Commands {
    Init(String),
    Build(String),
    Serve(String),
    New { name: String, post: bool },
}

pub fn parse_command(args: Vec<String>) -> Option<Commands> {
    let (cmd, rest) = args.split_first()?;
    let path = rest.get(0).map(|s| s.as_str()).unwrap_or(".");

    match cmd.as_str() {
        "build" => Some(Commands::Build(path.into())),
        "init" => Some(Commands::Init(path.into())),
        "serve" => Some(Commands::Serve(path.into())),
        "new" => {
            let name = rest.get(0)?.to_string();
            let post = rest.iter().any(|a| a == "--post");
            Some(Commands::New { name, post })
        }
        "-h" | "--help" => None,
        _ => None,
    }
}
