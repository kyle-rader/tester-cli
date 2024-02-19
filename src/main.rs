use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use rayon::prelude::*;

use clap::Parser;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {}

#[derive(Debug)]
enum Language {
    Rust,
    Elixir,
}

impl Language {
    fn detect(path: &Path) -> Option<Self> {
        if path.join("Cargo.toml").exists() {
            Some(Self::Rust)
        } else if path.join("mix.exs").exists() {
            Some(Self::Elixir)
        } else {
            None
        }
    }

    fn test_command(&self) -> Command {
        match self {
            Self::Rust => {
                let mut cmd = Command::new("cargo");
                cmd.args(["test"]);
                cmd
            }
            Self::Elixir => {
                let mut cmd = Command::new("mix");
                cmd.args(["test"]);
                cmd
            }
        }
    }
}

#[derive(Debug)]
struct Project {
    pub language: Language,
    pub root: PathBuf,
    pub name: String,
}

impl Project {
    fn detect(path: &Path) -> Option<Self> {
        Language::detect(path).map(|language| Self {
            language,
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            root: path.to_path_buf(),
        })
    }

    fn test(&self) -> anyhow::Result<String> {
        let mut command = self.language.test_command();
        let command = command.current_dir(&self.root).output()?;
        if command.status.success() {
            Ok(format!("✅ {}", self.name))
        } else {
            Ok(format!("❌ {}", self.name))
        }
    }
}

fn main() -> anyhow::Result<()> {
    let Args { .. } = Args::parse();

    let start_dir = env::current_dir()?;

    let mut projects = WalkDir::new(start_dir)
        .max_depth(1) // problems should all be in the current directory
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_dir())
        .skip(1) // Skips the current directory itself which is first in the list
        .filter_map(|e| Project::detect(e.path()))
        .collect::<Vec<_>>();

    projects.sort_by(|a, b| a.name.cmp(&b.name));

    projects.par_iter().for_each(|p| match p.test() {
        Ok(result) => println!("{result}"),
        Err(err) => eprintln!("💣 {}: {err}", p.name),
    });

    Ok(())
}
