//! Native character-wizard command-line adapter.

use std::{
    env, fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::character_wizard_domain::{Character, ResolvedCharacter};
use clap::{Args, Parser, Subcommand};

use crate::{
    app_error::{AppError, ErrorKind},
    character_wizard_creation, character_wizard_pdf_renderer, character_wizard_srd_data, data_pack,
    rules::RulesContext,
    share,
    storage::{self, CharacterRepository},
    template::resolve_template,
};

#[derive(Parser)]
#[command(about = "Create D&D characters using SRD 5.2.1.", version)]
struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "DIRECTORY",
        help = "Validated external campaign data pack directory"
    )]
    data: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Create(CreateArgs),
    Random(RandomArgs),
    Edit(EditArgs),
    Render(RenderArgs),
    List(ListArgs),
    Validate { character_json: PathBuf },
    Show(ShowArgs),
    Export(ExportArgs),
    Import(ImportArgs),
}

#[derive(Args)]
struct CreateArgs {
    #[arg(long)]
    template: Option<PathBuf>,
    #[arg(long, conflicts_with = "from_json")]
    quick: bool,
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
struct RandomArgs {
    #[arg(long = "class", value_name = "CLASS")]
    class_name: Option<String>,
    #[arg(long, value_name = "BACKGROUND")]
    background: Option<String>,
    #[arg(long, value_name = "SPECIES")]
    species: Option<String>,
    #[arg(long)]
    template: Option<PathBuf>,
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
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct EditArgs {
    #[command(flatten)]
    character: CharacterRefArgs,
    #[arg(long, help = "Render the edited character to this PDF path")]
    output: Option<PathBuf>,
    #[arg(long, requires = "output")]
    template: Option<PathBuf>,
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct RenderArgs {
    #[command(flatten)]
    character: CharacterRefArgs,
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

#[derive(Args)]
struct ShowArgs {
    #[command(flatten)]
    character: CharacterRefArgs,
}

#[derive(Args)]
struct ExportArgs {
    #[command(flatten)]
    character: CharacterRefArgs,
}

#[derive(Args)]
struct ImportArgs {
    #[arg(value_name = "CODE")]
    code: String,
    #[arg(
        long,
        value_name = "PATH",
        help = "JSON destination (defaults to the character collection)"
    )]
    output: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with = "output",
        help = "Character collection directory (defaults to the current directory)"
    )]
    directory: Option<PathBuf>,
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct CharacterRefArgs {
    #[arg(value_name = "CHARACTER")]
    character: PathBuf,
    #[arg(
        long,
        value_name = "PATH",
        help = "Character collection directory (defaults to the current directory)"
    )]
    directory: Option<PathBuf>,
}

#[derive(Args)]
struct ListArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Character collection directory (defaults to the current directory)"
    )]
    directory: Option<PathBuf>,
}

#[must_use]
pub fn main() -> ExitCode {
    let cli = Cli::parse_from(env::args_os());
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}

type CliResult = Result<(), AppError>;

fn run(cli: Cli) -> CliResult {
    let pack = cli
        .data
        .as_deref()
        .map(data_pack::load)
        .transpose()
        .map_err(|error| (1, error))?;
    let rules = match pack.as_ref() {
        Some(pack) => pack.rules(),
        None => RulesContext::srd(),
    };
    if let Some(reference) = rules.reference()
        && !matches!(&cli.command, Command::Export(_))
    {
        println!(
            "Using data pack: {} ({})",
            rules.name().unwrap_or(&reference.id),
            reference.id
        );
    }
    match cli.command {
        Command::Create(options) => create(options, rules),
        Command::Random(options) => random(options, rules),
        Command::Edit(options) => edit(options, rules),
        Command::Render(options) => render(options, rules),
        Command::List(options) => list(&options, rules),
        Command::Validate { character_json } => validate(&character_json, rules),
        Command::Show(options) => show(&resolve_character_path(&options.character), rules),
        Command::Export(options) => export_character(&options, rules),
        Command::Import(options) => import_character(options, rules),
    }
}

fn export_character(options: &ExportArgs, rules: RulesContext<'_>) -> CliResult {
    let character = load_character(&resolve_character_path(&options.character), rules)?;
    println!("{}", share::encode(&character).map_err(|error| (1, error))?);
    Ok(())
}

fn import_character(options: ImportArgs, rules: RulesContext<'_>) -> CliResult {
    let character = resolve_rules(
        share::decode(&options.code).map_err(|error| (1, error))?,
        rules,
    )?;
    let output = options.output.unwrap_or_else(|| {
        collection_directory(options.directory.as_deref())
            .join(character_output_path(&character.name, "json"))
    });
    if output.exists() && !options.force {
        return Err(AppError::new(
            ErrorKind::Input,
            format!(
                "import destination already exists: {}; pass --force to overwrite it",
                output.display()
            ),
        ));
    }
    write_character(&output, &character)?;
    println!("Imported {}.", character.name);
    println!("JSON: {}", output.display());
    Ok(())
}

fn random(options: RandomArgs, rules: RulesContext<'_>) -> CliResult {
    let requested_class = options.class_name.as_deref();
    let custom_class = requested_class.and_then(|value| rules.custom_class(value));
    let character_class = if custom_class.is_some() {
        requested_class.map(str::to_owned)
    } else {
        requested_class
            .map(|value| {
                canonical_srd_choice(value, &character_wizard_srd_data::CLASS_NAMES, "class")
            })
            .transpose()?
    };
    let requested_species = options.species.as_deref();
    let requested_background = options.background.as_deref();
    let custom_background = requested_background.and_then(|value| rules.custom_background(value));
    let background = if custom_background.is_some() {
        requested_background.map(str::to_owned)
    } else {
        requested_background
            .map(|value| {
                canonical_srd_choice(
                    value,
                    &character_wizard_srd_data::BACKGROUND_NAMES,
                    "background",
                )
            })
            .transpose()?
    };
    let custom_species = requested_species.and_then(|value| rules.custom_species(value));
    let species = if custom_species.is_some() {
        requested_species.map(str::to_owned)
    } else {
        requested_species
            .map(|value| {
                canonical_srd_choice(value, &character_wizard_srd_data::SPECIES_NAMES, "species")
            })
            .transpose()?
    };
    let template = resolve_template(options.template.as_deref()).map_err(|error| (1, error))?;
    let character = resolve_rules(
        character_wizard_creation::generate_random_character_with_rules(
            character_class.as_deref(),
            background.as_deref(),
            species.as_deref(),
            rules,
        )
        .map_err(|error| (1, error.to_string()))?,
        rules,
    )?;
    let json_output = options
        .json
        .unwrap_or_else(|| character_output_path(&character.name, "json"));
    let pdf_output = options
        .output
        .unwrap_or_else(|| character_output_path(&character.name, "pdf"));
    confirm_overwrite(&[&json_output, &pdf_output], options.force)?;
    create_parent(&pdf_output)?;
    write_character(&json_output, &character)?;
    if let Err(error) =
        character_wizard_pdf_renderer::render_character(&character, &template, &pdf_output)
    {
        let _ = fs::remove_file(&json_output);
        return Err(AppError::new(ErrorKind::Rendering, error));
    }
    println!("{} is ready!", character.name);
    println!("PDF: {}", pdf_output.display());
    println!("JSON: {}", json_output.display());
    Ok(())
}

fn list(options: &ListArgs, rules: RulesContext<'_>) -> CliResult {
    let directory = collection_directory(options.directory.as_deref());
    let characters = collection_characters(&directory, rules)?;
    if characters.is_empty() {
        println!("No characters found in {}.", directory.display());
        return Ok(());
    }
    println!("NAME\tCLASS\tLEVEL\tSPECIES");
    for character in characters {
        println!(
            "{}\t{}\t{}\t{}",
            character.name,
            character.class_name(),
            character.level,
            character.species_name()
        );
    }
    Ok(())
}

fn render(options: RenderArgs, rules: RulesContext<'_>) -> CliResult {
    let character = load_character(&resolve_character_path(&options.character), rules)?;
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

fn edit(options: EditArgs, rules: RulesContext<'_>) -> CliResult {
    let character_path = resolve_character_path(&options.character);
    let character = load_character(&character_path, rules)?;
    let Some(mut edited) =
        character_wizard_creation::run_edit_interactive_with_rules(&character, rules)
            .map_err(|error| (1, error.to_string()))?
    else {
        println!("No changes saved.");
        return Ok(());
    };
    edited.data_pack.clone_from(&character.data_pack);
    let edited = resolve_rules(edited, rules)?;
    let template = options
        .output
        .as_ref()
        .map(|_| resolve_template(options.template.as_deref()))
        .transpose()
        .map_err(|error| (1, error))?;
    let mut outputs = vec![character_path.as_path()];
    if let Some(output) = options.output.as_deref() {
        outputs.push(output);
    }
    confirm_overwrite(&outputs, options.force)?;
    write_character(&character_path, &edited)?;
    if let Some(output) = options.output {
        create_parent(&output)?;
        if let Err(error) = character_wizard_pdf_renderer::render_character(
            &edited,
            template
                .as_ref()
                .expect("template resolved when output is set"),
            &output,
        ) {
            return Err(AppError::new(ErrorKind::Rendering, error));
        }
        println!("PDF: {}", output.display());
    }
    println!("{} updated.", edited.name);
    println!("JSON: {}", character_path.display());
    Ok(())
}

fn validate(path: &Path, rules: RulesContext<'_>) -> CliResult {
    let character = load_character(path, rules)?;
    println!("{} is valid.", character.name);
    Ok(())
}

fn show(path: &Path, rules: RulesContext<'_>) -> CliResult {
    let character = load_character(path, rules)?;
    println!("{}", character.name);
    println!(
        "Identity      Level {} {} {}",
        character.level,
        character.species_name(),
        character.class_name()
    );
    println!("Background    {}", character.background_name());
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
    println!(
        "Equipment     {}",
        character
            .inventory()
            .into_iter()
            .map(|item| if item.quantity > 1 {
                format!("{} x {}", item.quantity, item.name)
            } else {
                item.name
            })
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("Gold          {} GP", character.coins().gold);
    Ok(())
}

fn create(options: CreateArgs, rules: RulesContext<'_>) -> CliResult {
    let template = resolve_template(options.template.as_deref()).map_err(|error| (1, error))?;

    let mut completed_draft = None;
    let mut character = if let Some(source) = options.from_json {
        load_character(&source, rules)?.into_record()
    } else if options.quick {
        character_wizard_creation::run_quick_interactive_with_rules(rules)
            .map_err(|error| (1, error.to_string()))?
    } else {
        let draft = options.draft;
        println!(
            "Progress is checkpointed in {}; Ctrl-C keeps the latest completed stage.",
            draft.display()
        );
        match character_wizard_creation::run_interactive_with_rules(&draft, rules) {
            Ok(character) => {
                completed_draft = Some(draft);
                character
            }
            Err(character_wizard_creation::WizardError::SaveAndExit) => {
                println!("Creation saved in {}.", draft.display());
                return Ok(());
            }
            Err(error) => return Err(AppError::new(ErrorKind::Input, error.to_string())),
        }
    };
    character.data_pack = rules.reference().cloned();
    let character = resolve_rules(character, rules)?;
    let json_output = options
        .json
        .unwrap_or_else(|| character_output_path(&character.name, "json"));
    let pdf_output = options
        .output
        .unwrap_or_else(|| character_output_path(&character.name, "pdf"));
    confirm_overwrite(&[&json_output, &pdf_output], options.force)?;
    create_parent(&pdf_output)?;
    write_character(&json_output, &character)?;
    if let Err(error) =
        character_wizard_pdf_renderer::render_character(&character, &template, &pdf_output)
    {
        let _ = fs::remove_file(&json_output);
        return Err(AppError::new(ErrorKind::Rendering, error));
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

fn canonical_srd_choice(value: &str, choices: &[&str], label: &str) -> CliResultValue<String> {
    choices
        .iter()
        .find(|choice| choice.eq_ignore_ascii_case(value))
        .map(|choice| (*choice).to_owned())
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::Input,
                format!(
                    "unknown SRD {label}: {value} (choose one of: {})",
                    choices.join(", ")
                ),
            )
        })
}

type CliResultValue<T> = Result<T, AppError>;

fn collection_directory(directory: Option<&Path>) -> PathBuf {
    CharacterRepository::new(directory)
        .directory()
        .to_path_buf()
}

fn resolve_character_path(character: &CharacterRefArgs) -> PathBuf {
    CharacterRepository::new(character.directory.as_deref()).resolve(&character.character)
}

fn collection_characters(
    directory: &Path,
    rules: RulesContext<'_>,
) -> Result<Vec<ResolvedCharacter>, AppError> {
    let paths = CharacterRepository::new(Some(directory))
        .json_paths()
        .map_err(|error| AppError::new(ErrorKind::Persistence, error))?;
    let mut characters = Vec::new();
    for path in paths {
        let source = read_character_source(&path)?;
        let Ok(character) = Character::from_json(&source) else {
            continue;
        };
        characters.push(resolve_rules(character, rules)?);
    }
    Ok(characters)
}

fn load_character(path: &Path, rules: RulesContext<'_>) -> Result<ResolvedCharacter, AppError> {
    let source = read_character_source(path)?;
    let character = Character::from_json(&source).map_err(|error| {
        AppError::new(
            ErrorKind::Input,
            format!("invalid character JSON {}: {error}", path.display()),
        )
    })?;
    resolve_rules(character, rules)
}

fn read_character_source(path: &Path) -> Result<String, AppError> {
    if !path.is_file() {
        return Err(AppError::new(
            ErrorKind::Input,
            format!(
                "character JSON does not exist or is not a file: {}",
                path.display()
            ),
        ));
    }
    fs::read_to_string(path).map_err(|error| {
        AppError::new(
            ErrorKind::Persistence,
            format!("unable to read {}: {error}", path.display()),
        )
    })
}

fn resolve_rules(
    character: Character,
    rules: RulesContext<'_>,
) -> Result<ResolvedCharacter, AppError> {
    rules.resolve(character).map_err(|error| {
        AppError::new(
            ErrorKind::Rules,
            rules.reference().map_or(error.clone(), |reference| {
                format!("{error} in data pack {}", reference.id)
            }),
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
        .map_err(|error| AppError::new(ErrorKind::Persistence, error.to_string()))?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| AppError::new(ErrorKind::Persistence, error.to_string()))?;
    if matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
        Ok(())
    } else {
        Err(AppError::new(ErrorKind::Input, "Aborted"))
    }
}

fn create_parent(path: &Path) -> CliResult {
    storage::create_parent(path).map_err(|error| AppError::new(ErrorKind::Persistence, error))
}

fn write_character(path: &Path, character: &Character) -> CliResult {
    let source = character.to_json().map_err(|error| (1, error))?;
    storage::write_atomic(path, source.as_bytes())
        .map_err(|error| AppError::new(ErrorKind::Persistence, error))
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

    use super::{
        CharacterRefArgs, Cli, Command, ImportArgs, RandomArgs, RenderArgs, RulesContext,
        canonical_srd_choice, character_output_path, collection_characters, import_character,
        load_character, random, render, resolve_character_path,
    };

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
        assert!(!options.quick);
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
    fn random_accepts_case_insensitive_constraints() {
        let cli = Cli::try_parse_from([
            "character-wizard",
            "random",
            "--class",
            "wizard",
            "--species",
            "dwarf",
        ])
        .expect("parse random");
        let Command::Random(options) = cli.command else {
            panic!("expected random command");
        };
        assert_eq!(options.class_name.as_deref(), Some("wizard"));
        assert_eq!(options.species.as_deref(), Some("dwarf"));
        assert_eq!(
            canonical_srd_choice(
                options.class_name.as_deref().expect("class"),
                &crate::character_wizard_srd_data::CLASS_NAMES,
                "class"
            ),
            Ok("Wizard".to_owned())
        );
    }

    #[test]
    fn create_accepts_quick_but_not_a_json_source() {
        let cli = Cli::try_parse_from(["character-wizard", "create", "--quick"])
            .expect("parse quick create");
        let Command::Create(options) = cli.command else {
            panic!("expected create command");
        };
        assert!(options.quick);
        assert!(
            Cli::try_parse_from([
                "character-wizard",
                "create",
                "--quick",
                "--from-json",
                "legolas.json",
            ])
            .is_err()
        );
    }

    #[test]
    fn global_data_pack_option_is_available_after_a_command() {
        let cli = Cli::try_parse_from(["character-wizard", "random", "--data", "my-campaign"])
            .expect("parse data pack option");
        assert_eq!(cli.data, Some(PathBuf::from("my-campaign")));
        assert!(matches!(cli.command, Command::Random(_)));
    }

    #[test]
    fn export_and_import_accept_collection_or_explicit_destinations() {
        let cli = Cli::try_parse_from([
            "character-wizard",
            "export",
            "legolas",
            "--directory",
            "party",
        ])
        .expect("parse export");
        let Command::Export(options) = cli.command else {
            panic!("expected export command");
        };
        assert_eq!(options.character.character, PathBuf::from("legolas"));
        assert_eq!(options.character.directory, Some(PathBuf::from("party")));

        let cli = Cli::try_parse_from([
            "character-wizard",
            "import",
            "cw1:AAAA",
            "--output",
            "party/legolas.json",
            "--force",
        ])
        .expect("parse import");
        let Command::Import(options) = cli.command else {
            panic!("expected import command");
        };
        assert_eq!(options.output, Some(PathBuf::from("party/legolas.json")));
        assert!(options.force);
    }

    #[test]
    fn import_writes_canonical_json_and_refuses_an_existing_destination() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let character = crate::character_wizard_domain::Character::from_json(include_str!(
            "../fixtures/complete-character.json"
        ))
        .expect("character fixture");
        let code = crate::share::encode(&character).expect("share code");
        let directory = std::env::temp_dir().join(format!(
            "character-wizard-import-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let output = directory.join("binary-smoke-test.json");
        import_character(
            ImportArgs {
                code: code.clone(),
                output: None,
                directory: Some(directory.clone()),
                force: false,
            },
            RulesContext::srd(),
        )
        .expect("import character");
        let imported =
            load_character(&output, RulesContext::srd()).expect("load imported character");
        assert_eq!(imported, character);
        let error = import_character(
            ImportArgs {
                code,
                output: None,
                directory: Some(directory.clone()),
                force: false,
            },
            RulesContext::srd(),
        )
        .expect_err("refuse collision");
        assert!(error.message().contains("--force"));
        std::fs::remove_file(output).expect("remove imported fixture");
        std::fs::remove_dir(directory).expect("remove import collection");
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
            options.character.character,
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
            options.character.character,
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
        render(
            RenderArgs {
                character: CharacterRefArgs {
                    character: PathBuf::from("fixtures/complete-character.json"),
                    directory: None,
                },
                template: Some(PathBuf::from("assets/character-sheet.pdf")),
                output: Some(output.clone()),
                force: true,
            },
            RulesContext::srd(),
        )
        .expect("render fixture");
        assert!(output.is_file());
        std::fs::remove_file(output).expect("remove rendered PDF");
    }

    #[test]
    fn random_pack_content_round_trips_and_renders_its_mechanics() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "character-wizard-pack-species-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).expect("create pack");
        std::fs::write(
            directory.join("data-pack.json"),
            r#"{"format_version":1,"id":"moon-pack","version":1,"name":"Moon Pack","files":{"species":"species.json","backgrounds":"backgrounds.json","equipment":"equipment.json"}}"#,
        )
        .expect("write manifest");
        std::fs::write(
            directory.join("species.json"),
            r#"[{"id":"moonfolk","name":"Moonfolk","sizes":["Small"],"speed":35,"darkvision_range":60,"traits":["Moonlit Step"]}]"#,
        )
        .expect("write species");
        std::fs::write(
            directory.join("backgrounds.json"),
            r#"[{"id":"lunar-scout","name":"Lunar Scout","abilities":["dexterity","wisdom","charisma"],"skills":["Perception","Survival"],"feat":"Alert","tool":"Navigator's Tools","equipment":[{"equipment_id":"moonblade"},{"name":"Arrow","quantity":20}],"equipment_gold":12,"gold_alternative":50}]"#,
        )
        .expect("write backgrounds");
        std::fs::write(
            directory.join("equipment.json"),
            r#"[{"id":"moonblade","name":"Moonblade","kind":{"type":"weapon","category":"Simple","kind":"Melee","properties":["Finesse","Light"],"mastery":"Vex","damage":"1d8","damage_type":"Radiant","normal_range":5}}]"#,
        )
        .expect("write equipment");
        let pack = crate::data_pack::load(&directory).expect("load pack");
        let json = directory.join("moonfolk.json");
        let pdf = directory.join("moonfolk.pdf");
        random(
            RandomArgs {
                class_name: Some("fighter".to_owned()),
                background: Some("lunar-scout".to_owned()),
                species: Some("moonfolk".to_owned()),
                template: Some(PathBuf::from("assets/character-sheet.pdf")),
                json: Some(json.clone()),
                output: Some(pdf.clone()),
                force: true,
            },
            pack.rules(),
        )
        .expect("generate pack species");

        assert!(
            load_character(&json, RulesContext::srd())
                .expect_err("pack reference is required")
                .message()
                .contains("requires data pack moon-pack")
        );
        let character = load_character(&json, pack.rules()).expect("reload pack character");
        assert_eq!(
            character
                .data_pack
                .as_ref()
                .expect("pack reference")
                .version,
            1
        );
        assert_eq!(character.species, "moonfolk");
        assert_eq!(character.background, "lunar-scout");
        assert_eq!(character.background_name(), "Lunar Scout");
        assert!(character.skills().contains("Perception"));
        assert!(
            character
                .all_tool_proficiencies()
                .contains(&"Navigator's Tools".to_owned())
        );
        assert_eq!(character.species_name(), "Moonfolk");
        assert_eq!(character.size, "Small");
        assert_eq!(character.speed(), 35);
        assert_eq!(character.darkvision_range(), Some(60));
        assert!(
            character
                .species_traits()
                .iter()
                .any(|value| value == "Moonlit Step")
        );
        let field = crate::character_wizard_pdf_renderer::read_field_value(&pdf, "Text8")
            .expect("read species field");
        assert_eq!(field.as_str().expect("text value"), b"Moonfolk");
        let field = crate::character_wizard_pdf_renderer::read_field_value(&pdf, "Text6")
            .expect("read background field");
        assert_eq!(field.as_str().expect("text value"), b"Lunar Scout");
        std::fs::remove_dir_all(directory).expect("remove pack");
    }

    #[test]
    fn a_bare_name_resolves_from_the_selected_collection_directory() {
        let character = CharacterRefArgs {
            character: PathBuf::from("legolas"),
            directory: Some(PathBuf::from("party")),
        };
        assert_eq!(
            resolve_character_path(&character),
            PathBuf::from("party/legolas.json")
        );
    }

    #[test]
    fn collection_listing_loads_json_characters_in_path_order() {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "character-wizard-collection-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).expect("create collection");
        std::fs::write(
            directory.join("rogue.json"),
            include_str!("../fixtures/complete-character.json"),
        )
        .expect("write character");
        std::fs::write(directory.join("notes.txt"), "not a character").expect("write note");
        std::fs::write(
            directory.join("package.json"),
            r#"{"name":"other-project"}"#,
        )
        .expect("write unrelated JSON");
        std::fs::write(directory.join("broken.json"), "{not json").expect("write malformed JSON");

        let characters =
            collection_characters(&directory, RulesContext::srd()).expect("load collection");
        std::fs::remove_dir_all(&directory).expect("remove collection");
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].name, "Binary Smoke Test");
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
