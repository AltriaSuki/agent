use anyhow::Result;
use crate::cli::GenerateCommands;
use colored::Colorize;
use process_generators::{Generator, GeneratedFile};
use process_generators::githooks::GitHooksGenerator;
use process_generators::cicd::CiCdGenerator;
use process_generators::makefile::MakefileGenerator;
use process_generators::ide::IdeGenerator;
use std::path::Path;

pub fn execute(command: &GenerateCommands) -> Result<()> {
    let cwd = std::env::current_dir()?;

    match command {
        GenerateCommands::GitHooks => run_generator(&GitHooksGenerator, &cwd),
        GenerateCommands::Cicd => run_generator(&CiCdGenerator, &cwd),
        GenerateCommands::Makefile => run_generator(&MakefileGenerator, &cwd),
        GenerateCommands::Ide => run_generator(&IdeGenerator, &cwd),
        GenerateCommands::All => {
            println!("{}", "Running all generators...".bold());
            let generators: Vec<Box<dyn Generator>> = vec![
                Box::new(GitHooksGenerator),
                Box::new(CiCdGenerator),
                Box::new(MakefileGenerator),
                Box::new(IdeGenerator),
            ];
            for gen in &generators {
                run_generator(gen.as_ref(), &cwd)?;
            }
            println!("\n{}", "✅ All generators complete".green().bold());
            Ok(())
        }
    }
}

fn run_generator(gen: &dyn Generator, project_root: &Path) -> Result<()> {
    println!("{} {} — {}", "▶".cyan(), gen.name().bold(), gen.description());
    let files = gen.generate(project_root)?;

    for file in &files {
        write_generated_file(project_root, file)?;
    }

    if files.is_empty() {
        println!("  (no files generated)");
    }

    Ok(())
}

fn write_generated_file(project_root: &Path, file: &GeneratedFile) -> Result<()> {
    let full_path = project_root.join(&file.path);

    // Create parent dirs
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&full_path, &file.content)?;

    // Set executable for hooks
    #[cfg(unix)]
    if file.path.contains("hooks/") {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&full_path, std::fs::Permissions::from_mode(0o755))?;
    }

    let status = if file.overwritten {
        "overwritten".yellow().to_string()
    } else {
        "created".green().to_string()
    };

    println!("  {} {}", status, file.path);
    Ok(())
}
