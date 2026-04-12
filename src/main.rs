mod browser;
mod config;
mod editor;
mod git;
mod iterm;
mod port;
mod process;
mod state;

use clap::{Parser, Subcommand};
use dialoguer::FuzzySelect;

#[derive(Parser)]
#[command(name = "launch", about = "One-command dev environment launcher")]
struct Cli {
    /// Project name to launch
    project: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Stop project services
    Stop {
        /// Project name to stop
        project: Option<String>,
        /// Stop all running projects
        #[arg(long)]
        all: bool,
    },
    /// List all projects and their status
    List,
    /// Edit project config in $EDITOR
    Edit {
        /// Project name
        project: String,
    },
    /// Create new project config from template
    New {
        /// Project name
        project: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Stop { project, all }) => {
            if all {
                process::stop_all()
            } else if let Some(name) = project {
                process::stop(&name)
            } else {
                Err("Usage: launch stop <project> or launch stop --all".to_string())
            }
        }
        Some(Commands::List) => process::list(),
        Some(Commands::Edit { project }) => process::edit(&project),
        Some(Commands::New { project }) => process::new_project(&project),
        None => {
            if let Some(name) = cli.project {
                process::launch(&name)
            } else {
                fuzzy_select()
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn fuzzy_select() -> Result<(), String> {
    config::ensure_dirs().map_err(|e| e.to_string())?;
    let projects = config::list_projects();
    if projects.is_empty() {
        return Err("No projects configured. Run `launch new <name>` to create one.".to_string());
    }

    let selection = FuzzySelect::new()
        .with_prompt("Select project")
        .items(&projects)
        .interact_opt()
        .map_err(|e| format!("Selection error: {e}"))?;

    if let Some(idx) = selection {
        process::launch(&projects[idx])
    } else {
        println!("Cancelled.");
        Ok(())
    }
}
