//! Native character-wizard command-line entry point.

use std::{
    env, fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::character_wizard_domain::Character;
use clap::{Args, Parser, Subcommand};

pub mod creation;
pub mod domain;
pub mod pdf_renderer;
pub mod srd_data;

pub use self::creation as character_wizard_creation;
pub use self::domain as character_wizard_domain;
pub use self::pdf_renderer as character_wizard_pdf_renderer;
pub use self::srd_data as character_wizard_srd_data;

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
    Edit(EditArgs),
    Render(RenderArgs),
    Validate { character_json: PathBuf },
    Show { character_json: PathBuf },
}

#[derive(Args)]
struct CreateArgs {
    #[arg(long)]
    template: Option<PathBuf>,
    #[arg(long)]
    from_json: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Path for the character JSON (defaults to <character-name>.json)"
    )]
    json: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Path for the filled character sheet (defaults to <character-name>.pdf)"
    )]
    output: Option<PathBuf>,
    #[arg(long, default_value = "character-draft.json")]
    draft: PathBuf,
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct EditArgs {
    #[arg(value_name = "CHARACTER_JSON")]
    character_json: PathBuf,
    #[arg(long, help = "Render the edited character to this PDF path")]
    output: Option<PathBuf>,
    #[arg(long, requires = "output")]
    template: Option<PathBuf>,
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct RenderArgs {
    #[arg(value_name = "CHARACTER_JSON")]
    character_json: PathBuf,
    #[arg(long)]
    template: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Path for the filled character sheet (defaults to <character-name>.pdf)"
    )]
    output: Option<PathBuf>,
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
        Command::Edit(options) => edit(options),
        Command::Render(options) => render(options),
        Command::Validate { character_json } => validate(&character_json),
        Command::Show { character_json } => show(&character_json),
    }
}

fn render(options: RenderArgs) -> CliResult {
    let character = load_character(&options.character_json)?;
    let template = resolve_template(options.template.as_deref()).map_err(|error| (1, error))?;
    let output = options
        .output
        .unwrap_or_else(|| character_output_path(&character.name, "pdf"));
    confirm_overwrite(&[&output], options.force)?;
    create_parent(&output)?;
    character_wizard_pdf_renderer::render_character(&character, &template, &output)
        .map_err(|error| (1, error))?;
    println!("PDF: {}", output.display());
    Ok(())
}

fn edit(options: EditArgs) -> CliResult {
    let character = load_character(&options.character_json)?;
    let Some(edited) = character_wizard_creation::run_edit_interactive(&character)
        .map_err(|error| (1, error.to_string()))?
    else {
        println!("No changes saved.");
        return Ok(());
    };
    let template = options
        .output
        .as_ref()
        .map(|_| resolve_template(options.template.as_deref()))
        .transpose()
        .map_err(|error| (1, error))?;
    let mut outputs = vec![options.character_json.as_path()];
    if let Some(output) = options.output.as_deref() {
        outputs.push(output);
    }
    confirm_overwrite(&outputs, options.force)?;
    create_parent(&options.character_json)?;
    fs::write(
        &options.character_json,
        edited.to_json().map_err(|error| (1, error))?,
    )
    .map_err(|error| {
        (
            1,
            format!(
                "unable to write {}: {error}",
                options.character_json.display()
            ),
        )
    })?;
    if let Some(output) = options.output {
        create_parent(&output)?;
        if let Err(error) = character_wizard_pdf_renderer::render_character(
            &edited,
            template
                .as_ref()
                .expect("template resolved when output is set"),
            &output,
        ) {
            return Err((1, error));
        }
        println!("PDF: {}", output.display());
    }
    println!("{} updated.", edited.name);
    println!("JSON: {}", options.character_json.display());
    Ok(())
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
    let json_output = options
        .json
        .unwrap_or_else(|| character_output_path(&character.name, "json"));
    let pdf_output = options
        .output
        .unwrap_or_else(|| character_output_path(&character.name, "pdf"));
    confirm_overwrite(&[&json_output, &pdf_output], options.force)?;
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

fn character_output_path(name: &str, extension: &str) -> PathBuf {
    let stem = name
        .trim()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let stem = stem.trim_matches('-');
    let stem = if stem.is_empty() { "character" } else { stem };
    PathBuf::from(format!("{stem}.{extension}"))
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
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use clap::Parser as _;

    use super::{Cli, Command, RenderArgs, character_output_path, render};

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
        assert!(options.json.is_none());
        assert!(options.output.is_none());
    }

    #[test]
    fn create_accepts_explicit_output_paths() {
        let cli = Cli::try_parse_from([
            "character-wizard",
            "create",
            "--json",
            "records/legolas.json",
            "--output",
            "sheets/legolas.pdf",
        ])
        .expect("parse create");
        let Command::Create(options) = cli.command else {
            panic!("expected create command");
        };
        assert_eq!(options.json, Some(PathBuf::from("records/legolas.json")));
        assert_eq!(options.output, Some(PathBuf::from("sheets/legolas.pdf")));
    }

    #[test]
    fn edit_accepts_an_optional_pdf_output() {
        let cli = Cli::try_parse_from([
            "character-wizard",
            "edit",
            "records/legolas.json",
            "--template",
            "assets/character-sheet.pdf",
            "--output",
            "sheets/legolas.pdf",
            "--force",
        ])
        .expect("parse edit");
        let Command::Edit(options) = cli.command else {
            panic!("expected edit command");
        };
        assert_eq!(
            options.character_json,
            PathBuf::from("records/legolas.json")
        );
        assert_eq!(
            options.template,
            Some(PathBuf::from("assets/character-sheet.pdf"))
        );
        assert_eq!(options.output, Some(PathBuf::from("sheets/legolas.pdf")));
        assert!(options.force);
    }

    #[test]
    fn render_accepts_explicit_paths() {
        let cli = Cli::try_parse_from([
            "character-wizard",
            "render",
            "records/legolas.json",
            "--template",
            "assets/character-sheet.pdf",
            "--output",
            "sheets/legolas.pdf",
            "--force",
        ])
        .expect("parse render");
        let Command::Render(options) = cli.command else {
            panic!("expected render command");
        };
        assert_eq!(
            options.character_json,
            PathBuf::from("records/legolas.json")
        );
        assert_eq!(
            options.template,
            Some(PathBuf::from("assets/character-sheet.pdf"))
        );
        assert_eq!(options.output, Some(PathBuf::from("sheets/legolas.pdf")));
        assert!(options.force);
    }

    #[test]
    fn render_writes_a_pdf_for_a_valid_character() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let output = std::env::temp_dir().join(format!(
            "character-wizard-render-test-{}-{}.pdf",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        render(RenderArgs {
            character_json: PathBuf::from("fixtures/complete-character.json"),
            template: Some(PathBuf::from("assets/character-sheet.pdf")),
            output: Some(output.clone()),
            force: true,
        })
        .expect("render fixture");
        assert!(output.is_file());
        std::fs::remove_file(output).expect("remove rendered PDF");
    }

    #[test]
    fn character_name_becomes_safe_default_output_name() {
        assert_eq!(
            character_output_path("Legolas", "json"),
            PathBuf::from("legolas.json")
        );
        assert_eq!(
            character_output_path("Aelinor of Rivendell", "pdf"),
            PathBuf::from("aelinor-of-rivendell.pdf")
        );
        assert_eq!(
            character_output_path("../../", "json"),
            PathBuf::from("character.json")
        );
    }
}
