//! Native character-wizard command-line entry point.

use std::{
    env, fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::character_wizard_domain::{Character, DataPackReference};
use clap::{Args, Parser, Subcommand};

pub mod creation;
mod data_pack;
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
struct CharacterRefArgs {
    #[arg(value_name = "CHARACTER")]
    character: PathBuf,
    #[arg(
        long,
        value_name = "PATH",
        help = "Character collection directory (defaults to ./characters)"
    )]
    directory: Option<PathBuf>,
}

#[derive(Args)]
struct ListArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Character collection directory (defaults to ./characters)"
    )]
    directory: Option<PathBuf>,
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
    let pack = cli
        .data
        .as_deref()
        .map(data_pack::load)
        .transpose()
        .map_err(|error| (1, error))?;
    if let Some(pack) = &pack {
        println!("Using data pack: {} ({})", pack.name, pack.id);
    }
    match cli.command {
        Command::Create(options) => create(options, pack.as_ref()),
        Command::Random(options) => random(options, pack.as_ref()),
        Command::Edit(options) => edit(options, pack.as_ref()),
        Command::Render(options) => render(options, pack.as_ref()),
        Command::List(options) => list(&options, pack.as_ref()),
        Command::Validate { character_json } => validate(&character_json, pack.as_ref()),
        Command::Show(options) => show(&resolve_character_path(&options.character), pack.as_ref()),
    }
}

fn random(options: RandomArgs, pack: Option<&data_pack::DataPackManifest>) -> CliResult {
    let requested_class = options.class_name.as_deref();
    let custom_class = requested_class.and_then(|value| {
        pack.and_then(|pack| {
            pack.classes.iter().find(|rule| {
                rule.id.eq_ignore_ascii_case(value) || rule.name.eq_ignore_ascii_case(value)
            })
        })
    });
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
    let custom_background = requested_background.and_then(|value| {
        pack.and_then(|pack| {
            pack.backgrounds.iter().find(|rule| {
                rule.id.eq_ignore_ascii_case(value) || rule.name.eq_ignore_ascii_case(value)
            })
        })
    });
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
    let custom_species = requested_species.and_then(|value| {
        pack.and_then(|pack| {
            pack.species.iter().find(|rule| {
                rule.id.eq_ignore_ascii_case(value) || rule.name.eq_ignore_ascii_case(value)
            })
        })
    });
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
    let character = character_wizard_creation::generate_random_character_with_pack(
        character_class.as_deref(),
        background.as_deref(),
        species.as_deref(),
        pack.map(data_pack_reference),
        pack.map_or(&[], |pack| pack.classes.as_slice()),
        pack.map_or(&[], |pack| pack.species.as_slice()),
        pack.map_or(&[], |pack| pack.backgrounds.as_slice()),
        pack.map_or(&[], |pack| pack.equipment.as_slice()),
        pack.map_or(&[], |pack| pack.spells.as_slice()),
    )
    .map_err(|error| (1, error.to_string()))?;
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
    Ok(())
}

const DEFAULT_CHARACTER_DIRECTORY: &str = "characters";

fn list(options: &ListArgs, pack: Option<&data_pack::DataPackManifest>) -> CliResult {
    let directory = collection_directory(options.directory.as_deref());
    let characters = collection_characters(&directory, pack)?;
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

fn render(options: RenderArgs, pack: Option<&data_pack::DataPackManifest>) -> CliResult {
    let character = load_character(&resolve_character_path(&options.character), pack)?;
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

fn edit(options: EditArgs, pack: Option<&data_pack::DataPackManifest>) -> CliResult {
    let character_path = resolve_character_path(&options.character);
    let character = load_character(&character_path, pack)?;
    let Some(mut edited) = character_wizard_creation::run_edit_interactive_with_pack(
        &character,
        pack.map_or(&[], |pack| pack.classes.as_slice()),
        pack.map_or(&[], |pack| pack.species.as_slice()),
        pack.map_or(&[], |pack| pack.backgrounds.as_slice()),
        pack.map_or(&[], |pack| pack.equipment.as_slice()),
        pack.map_or(&[], |pack| pack.spells.as_slice()),
    )
    .map_err(|error| (1, error.to_string()))?
    else {
        println!("No changes saved.");
        return Ok(());
    };
    edited.data_pack.clone_from(&character.data_pack);
    resolve_pack_content(&mut edited, pack)?;
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
    create_parent(&character_path)?;
    fs::write(
        &character_path,
        edited.to_json().map_err(|error| (1, error))?,
    )
    .map_err(|error| {
        (
            1,
            format!("unable to write {}: {error}", character_path.display()),
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
    println!("JSON: {}", character_path.display());
    Ok(())
}

fn validate(path: &Path, pack: Option<&data_pack::DataPackManifest>) -> CliResult {
    let character = load_character(path, pack)?;
    println!("{} is valid.", character.name);
    Ok(())
}

fn show(path: &Path, pack: Option<&data_pack::DataPackManifest>) -> CliResult {
    let character = load_character(path, pack)?;
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

fn create(options: CreateArgs, pack: Option<&data_pack::DataPackManifest>) -> CliResult {
    let template = resolve_template(options.template.as_deref()).map_err(|error| (1, error))?;
    let pack_reference = pack.map(data_pack_reference);

    let mut completed_draft = None;
    let mut character = if let Some(source) = options.from_json {
        load_character(&source, pack)?
    } else if options.quick {
        character_wizard_creation::run_quick_interactive_with_pack(
            pack_reference.as_ref(),
            pack.map_or(&[], |pack| pack.classes.as_slice()),
            pack.map_or(&[], |pack| pack.species.as_slice()),
            pack.map_or(&[], |pack| pack.backgrounds.as_slice()),
            pack.map_or(&[], |pack| pack.equipment.as_slice()),
            pack.map_or(&[], |pack| pack.spells.as_slice()),
        )
        .map_err(|error| (1, error.to_string()))?
    } else {
        let draft = options.draft;
        println!(
            "Progress is checkpointed in {}; Ctrl-C keeps the latest completed stage.",
            draft.display()
        );
        match character_wizard_creation::run_interactive_with_pack(
            &draft,
            pack_reference.clone(),
            pack.map_or(&[], |pack| pack.classes.as_slice()),
            pack.map_or(&[], |pack| pack.species.as_slice()),
            pack.map_or(&[], |pack| pack.backgrounds.as_slice()),
            pack.map_or(&[], |pack| pack.equipment.as_slice()),
            pack.map_or(&[], |pack| pack.spells.as_slice()),
        ) {
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
    set_data_pack(&mut character, pack);
    resolve_pack_content(&mut character, pack)?;
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

fn canonical_srd_choice(value: &str, choices: &[&str], label: &str) -> CliResultValue<String> {
    choices
        .iter()
        .find(|choice| choice.eq_ignore_ascii_case(value))
        .map(|choice| (*choice).to_owned())
        .ok_or_else(|| {
            (
                1,
                format!(
                    "unknown SRD {label}: {value} (choose one of: {})",
                    choices.join(", ")
                ),
            )
        })
}

type CliResultValue<T> = Result<T, (u8, String)>;

fn collection_directory(directory: Option<&Path>) -> PathBuf {
    directory.map_or_else(
        || PathBuf::from(DEFAULT_CHARACTER_DIRECTORY),
        Path::to_path_buf,
    )
}

fn resolve_character_path(character: &CharacterRefArgs) -> PathBuf {
    if character.character.is_file()
        || character.character.extension().is_some()
        || character.character.components().count() > 1
    {
        return character.character.clone();
    }
    collection_directory(character.directory.as_deref())
        .join(&character.character)
        .with_extension("json")
}

fn collection_characters(
    directory: &Path,
    pack: Option<&data_pack::DataPackManifest>,
) -> Result<Vec<Character>, (u8, String)> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    if !directory.is_dir() {
        return Err((
            1,
            format!(
                "character collection is not a directory: {}",
                directory.display()
            ),
        ));
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|error| {
            (
                1,
                format!("unable to read {}: {error}", directory.display()),
            )
        })?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            (
                1,
                format!("unable to read {}: {error}", directory.display()),
            )
        })?
        .into_iter()
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| load_character(&path, pack))
        .collect()
}

fn load_character(
    path: &Path,
    pack: Option<&data_pack::DataPackManifest>,
) -> Result<Character, (u8, String)> {
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
    let mut character = Character::from_json(&source).map_err(|error| {
        (
            1,
            format!("invalid character JSON {}: {error}", path.display()),
        )
    })?;
    ensure_data_pack(&character, pack)?;
    resolve_pack_content(&mut character, pack)?;
    Ok(character)
}

fn set_data_pack(character: &mut Character, pack: Option<&data_pack::DataPackManifest>) {
    character.data_pack = pack.map(data_pack_reference);
}

fn data_pack_reference(pack: &data_pack::DataPackManifest) -> DataPackReference {
    DataPackReference {
        id: pack.id.clone(),
        format_version: pack.format_version,
        version: pack.version,
    }
}

fn ensure_data_pack(
    character: &Character,
    pack: Option<&data_pack::DataPackManifest>,
) -> CliResult {
    let Some(reference) = &character.data_pack else {
        return Ok(());
    };
    let Some(pack) = pack else {
        return Err((
            1,
            format!(
                "character requires data pack {}; pass --data <directory>",
                reference.id
            ),
        ));
    };
    if reference.id != pack.id
        || reference.format_version != pack.format_version
        || reference.version != pack.version
    {
        return Err((
            1,
            format!(
                "character requires data pack {} version {} (format version {})",
                reference.id, reference.version, reference.format_version
            ),
        ));
    }
    Ok(())
}

fn resolve_pack_content(
    character: &mut Character,
    pack: Option<&data_pack::DataPackManifest>,
) -> CliResult {
    character
        .resolve_pack_class(pack.map_or(&[], |pack| pack.classes.as_slice()))
        .map_err(|error| {
            (
                1,
                pack.map_or(error.clone(), |pack| {
                    format!("{error} in data pack {}", pack.id)
                }),
            )
        })?;
    character
        .resolve_pack_background(pack.map_or(&[], |pack| pack.backgrounds.as_slice()))
        .map_err(|error| {
            (
                1,
                pack.map_or(error.clone(), |pack| {
                    format!("{error} in data pack {}", pack.id)
                }),
            )
        })?;
    character
        .resolve_pack_equipment(pack.map_or(&[], |pack| pack.equipment.as_slice()))
        .map_err(|error| {
            (
                1,
                pack.map_or(error.clone(), |pack| {
                    format!("{error} in data pack {}", pack.id)
                }),
            )
        })?;
    character
        .resolve_pack_spells(pack.map_or(&[], |pack| pack.spells.as_slice()))
        .map_err(|error| {
            (
                1,
                pack.map_or(error.clone(), |pack| {
                    format!("{error} in data pack {}", pack.id)
                }),
            )
        })?;
    if character_wizard_srd_data::species_rule(&character.species).is_some() {
        character
            .resolve_pack_species(&[])
            .map_err(|error| (1, error))?;
        return Ok(());
    }
    let pack = pack.ok_or_else(|| {
        (
            1,
            format!(
                "pack species {} requires --data <directory>",
                character.species
            ),
        )
    })?;
    character
        .resolve_pack_species(&pack.species)
        .map_err(|error| (1, format!("{error} in data pack {}", pack.id)))
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

    use super::{
        CharacterRefArgs, Cli, Command, RandomArgs, RenderArgs, canonical_srd_choice,
        character_output_path, collection_characters, load_character, random, render,
        resolve_character_path,
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
            None,
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
            Some(&pack),
        )
        .expect("generate pack species");

        assert!(
            load_character(&json, None)
                .expect_err("pack reference is required")
                .1
                .contains("requires data pack moon-pack")
        );
        let character = load_character(&json, Some(&pack)).expect("reload pack character");
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

        let characters = collection_characters(&directory, None).expect("load collection");
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
