#[derive(Debug)]
pub enum Commands {
    Init(String),
    Build(String),
    Serve { path: String, watch: bool },
    New { name: String, post: bool },
}

pub fn parse_command(args: Vec<String>) -> Option<Commands> {
    let (cmd, rest) = args.split_first()?;
    let path = rest
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or(".");

    match cmd.as_str() {
        "build" => Some(Commands::Build(path.into())),
        "init" => Some(Commands::Init(path.into())),
        "serve" => {
            let watch = rest.iter().any(|a| a == "--watch");
            Some(Commands::Serve { path: path.into(), watch })
        }
        "new" => {
            let name = rest.get(0)?.to_string();
            let post = rest.iter().any(|a| a == "--post");
            Some(Commands::New { name, post })
        }
        "-h" | "--help" => None,
        _ => None,
    }
}
