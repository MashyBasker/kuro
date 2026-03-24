#[derive(Debug)]
pub enum Commands {
    Init(String),
    Build(String),
    Serve(String),
}

pub fn parse_command(args: Vec<String>) -> Option<Commands> {
    let (cmd, rest) = args.split_first()?;
    let path = rest.get(0).map(|s| s.as_str()).unwrap_or(".");

    match cmd.as_str() {
        "build" => Some(Commands::Build(path.into())),
        "init" => Some(Commands::Init(path.into())),
        "serve" => Some(Commands::Serve(path.into())),
        "-h" | "--help" => None,
        _ => None,
    }
}
