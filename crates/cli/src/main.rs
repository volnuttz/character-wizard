//! Native character-wizard command-line entry point.

use std::{
    env, fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
    process::ExitCode,
};

use character_wizard_domain::Character;
use clap::{Args, Parser, Subcommand};

mod template;

use template::resolve_template;

#[derive(Parser)]
#[command(about = "Create D&D characters using SRD 5.2.1.", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Create(CreateArgs),
    Validate { character_json: PathBuf },
    Show { character_json: PathBuf },
}

#[derive(Args)]
struct CreateArgs {
    #[arg(long)]
    template: Option<PathBuf>,
    #[arg(long)]
    from_json: Option<PathBuf>,
    #[arg(long, default_value = "character.json")]
    json: PathBuf,
    #[arg(short, long, default_value = "character-sheet-filled.pdf")]
    output: PathBuf,
    #[arg(long, default_value = "character-draft.json")]
    draft: PathBuf,
    #[arg(long)]
    force: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse_from(env::args_os());
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            println!("Error: {message}");
            ExitCode::from(code)
        }
    }
}

type CliResult = Result<(), (u8, String)>;

fn run(cli: Cli) -> CliResult {
    match cli.command {
        Command::Create(options) => create(options),
        Command::Validate { character_json } => validate(&character_json),
        Command::Show { character_json } => show(&character_json),
    }
}

fn validate(path: &Path) -> CliResult {
    let character = load_character(path)?;
    println!("{} is valid.", character.name);
    Ok(())
}

fn show(path: &Path) -> CliResult {
    let character = load_character(path)?;
    println!("{}", character.name);
    println!(
        "Identity      Level {} {} {}",
        character.level, character.species, character.character_class
    );
    println!("Background    {}", character.background);
    println!("Alignment     {}", character.alignment);
    println!(
        "Combat        HP {} · AC {} · Speed {} ft.",
        character.hit_points(),
        character.armor_class(),
        character.speed()
    );
    println!(
        "Skills        {}",
        character
            .skills()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("Languages     {}", languages(&character).join(", "));
    Ok(())
}

fn create(options: CreateArgs) -> CliResult {
    let template = resolve_template(options.template.as_deref()).map_err(|error| (1, error))?;
    let json_output = options.json;
    let pdf_output = options.output;
    confirm_overwrite(&[&json_output, &pdf_output], options.force)?;

    let mut completed_draft = None;
    let character = if let Some(source) = options.from_json {
        load_character(&source)?
    } else {
        let draft = options.draft;
        println!(
            "Progress is checkpointed in {}; Ctrl-C keeps the latest completed stage.",
            draft.display()
        );
        match character_wizard_creation::run_interactive(&draft) {
            Ok(character) => {
                completed_draft = Some(draft);
                character
            }
            Err(character_wizard_creation::WizardError::SaveAndExit) => {
                println!("Creation saved in {}.", draft.display());
                return Ok(());
            }
            Err(error) => return Err((1, error.to_string())),
        }
    };
    create_parent(&json_output)?;
    create_parent(&pdf_output)?;
    fs::write(
        &json_output,
        character.to_json().map_err(|error| (1, error))?,
    )
    .map_err(|error| {
        (
            1,
            format!("unable to write {}: {error}", json_output.display()),
        )
    })?;
    if let Err(error) =
        character_wizard_pdf_renderer::render_character(&character, &template, &pdf_output)
    {
        let _ = fs::remove_file(&json_output);
        return Err((1, error));
    }
    println!("{} is ready!", character.name);
    println!("PDF: {}", pdf_output.display());
    println!("JSON: {}", json_output.display());
    if let Some(draft) = completed_draft {
        let _ = fs::remove_file(draft);
    }
    Ok(())
}

fn load_character(path: &Path) -> Result<Character, (u8, String)> {
    if !path.is_file() {
        return Err((
            1,
            format!(
                "character JSON does not exist or is not a file: {}",
                path.display()
            ),
        ));
    }
    let source = fs::read_to_string(path)
        .map_err(|error| (1, format!("unable to read {}: {error}", path.display())))?;
    Character::from_json(&source).map_err(|error| {
        (
            1,
            format!("invalid character JSON {}: {error}", path.display()),
        )
    })
}

fn confirm_overwrite(paths: &[&Path], force: bool) -> CliResult {
    let existing: Vec<String> = paths
        .iter()
        .filter(|path| path.exists())
        .map(|path| path.display().to_string())
        .collect();
    if existing.is_empty() || force {
        return Ok(());
    }
    print!(
        "Overwrite existing output(s): {}? [y/N] ",
        existing.join(", ")
    );
    io::stdout()
        .flush()
        .map_err(|error| (1, error.to_string()))?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| (1, error.to_string()))?;
    if matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        Ok(())
    } else {
        Err((1, "Aborted".to_owned()))
    }
}

fn create_parent(path: &Path) -> CliResult {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| (1, format!("unable to create {}: {error}", parent.display())))?;
    }
    Ok(())
}

fn languages(character: &Character) -> Vec<String> {
    let mut values = vec!["Common".to_owned()];
    values.extend(character.selected_languages.iter().cloned());
    values.extend(character.class_choices.additional_language.iter().cloned());
    values
}

#[cfg(test)]
mod tests {
    use clap::Parser as _;

    use super::{Cli, Command};

    #[test]
    fn clap_accepts_the_version_flag() {
        let error = match Cli::try_parse_from(["character-wizard", "--version"]) {
            Ok(_) => panic!("--version should display the package version"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
        assert!(error.to_string().contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn create_does_not_require_a_template_argument() {
        let cli = Cli::try_parse_from(["character-wizard", "create"]).expect("parse create");
        let Command::Create(options) = cli.command else {
            panic!("expected create command");
        };
        assert!(options.template.is_none());
    }
}
