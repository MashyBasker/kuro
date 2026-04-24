# Kuro

A static-site generator built to power my website and blog. Kuro is highly opinionated in it's file structure and layout.

## Features

- GitHub flavored markdown rendering
- Hot reloading in watch mode
- Uses the [the-monospace-web](https://owickstrom.github.io/the-monospace-web/) theme by [owickstrom](https://x.com/owickstrom) (with some variations)


## Installation

```shell
git clone https://github.com/mashybasker/kuro
cd kuro
chmod +x install.sh
./install.sh
```

`kuro` is installed to `~/.local/bin` by default.

## Quickstart

```shell
# Create a new project
kuro init mysite

# Build and serve the site
# Visit the site on localhost:3000
kuro serve
```

To incrementally build the site in watch mode with hot-reload, use the command:

```shell
kuro serve --watch
```

To check for available commands, run `kuro help`

```

 kuro 0.1.0

 USAGE:
     kuro <COMMAND> [OPTIONS]

 COMMANDS:
     init [PATH]       Create a new site (default: current directory)
     build [PATH]      Build the site (default: current directory)
     serve [PATH]      Serve the site locally (default: current directory)
     new <NAME>        Create a new page

 OPTIONS:
     --post            Create as a blog post (used with `new`)
     -h, --help        Print this help message

 EXAMPLES:
     kuro init                  Create a new site in the current directory
     kuro init my-site          Create a new site in ./my-site
     kuro build                 Build the site
     kuro new about             Create a new page called 'about'
     kuro new my-post --post    Create a new blog post
```

## Site structure

```shell
mysite/
├── content/ # source files for the site
├── dist/    # built site from the source markdown files
├── kuro.yml # config file for the project
├── static/  # static assets (eg. theme files)
└── templates/ # template html files
```

## Configuration

Edit `kuro.yml` to configure navbar links and social links in the footer.

```yaml
name: "My site"
author: "Your name"
description: "New Kuro Website"
url: "http://localhost:3000"

footer_links:
    - label: github
      url: https://github.com/<profile>

site_paths:
    - label: about
      path: /about
```

