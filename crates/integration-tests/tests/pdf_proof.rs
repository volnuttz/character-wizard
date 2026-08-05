use std::{fs, path::PathBuf};

use character_wizard_domain::Character;
use character_wizard_pdf_renderer::{
    acroform_inventory, read_field_values, render_character, template_inventory,
};
use lopdf::{Document, Object};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("root")
}

fn appearance_content(path: &std::path::Path, field_name: &str) -> String {
    let document = Document::load(path).expect("rendered PDF");
    for page_id in document.get_pages().into_values() {
        let page = document
            .get_object(page_id)
            .and_then(Object::as_dict)
            .expect("page");
        let (_, annotations) = document
            .dereference(page.get(b"Annots").expect("annotations"))
            .expect("annotation array");
        for annotation in annotations.as_array().expect("annotation array") {
            let widget_id = annotation.as_reference().expect("widget reference");
            let widget = document
                .get_object(widget_id)
                .and_then(Object::as_dict)
                .expect("widget");
            let name = widget
                .get(b"T")
                .ok()
                .and_then(|value| value.as_str().ok())
                .map(|value| String::from_utf8_lossy(value).into_owned());
            if name.as_deref() != Some(field_name) {
                continue;
            }
            let (_, appearance) = document
                .dereference(
                    widget
                        .get(b"AP")
                        .and_then(Object::as_dict)
                        .and_then(|value| value.get(b"N"))
                        .expect("normal appearance"),
                )
                .expect("appearance stream");
            return String::from_utf8_lossy(
                &appearance.as_stream().expect("appearance stream").content,
            )
            .into_owned();
        }
    }
    panic!("field appearance not found: {field_name}");
}

#[test]
fn renderer_enumerates_the_supported_template_before_writing() {
    let actual = template_inventory(repository_root().join("assets/character-sheet.pdf"))
        .expect("inventory");

    assert_eq!(actual.page_count, 2);
    assert_eq!(actual.field_count, 244);
    assert_eq!(actual.text_field_count, 170);
    assert_eq!(actual.button_field_count, 74);
    assert_eq!(actual.untyped_field_count, 0);
    assert_eq!(
        actual.sha256,
        "85549528e2df28eeff96a6e447a5ce19ae6b9bd0c836446cf09bdd63476b194d"
    );
}

#[test]
fn renderer_enumerates_the_complete_acroform_catalog() {
    let actual =
        acroform_inventory(repository_root().join("assets/character-sheet.pdf")).expect("catalog");

    assert_eq!(actual.page_count, 2);
    assert_eq!(actual.field_count, 425);
    assert_eq!(actual.text_field_count, 260);
    assert_eq!(actual.button_field_count, 151);
    assert_eq!(actual.untyped_field_count, 14);
    assert_eq!(
        actual.sha256,
        "211ab47d8428008778f7688b48cb69be264810d147685040e0d774ddaaa30b49"
    );
}

#[test]
fn production_renderer_writes_readable_values_and_appearances() {
    let root = repository_root();
    let directory = std::env::temp_dir().join(format!(
        "character-wizard-production-proof-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("temporary directory");
    let source = fs::read_to_string(root.join("fixtures/complete-character.json"))
        .expect("character fixture");
    let character = Character::from_json(&source).expect("valid character");
    let output = directory.join("sheet.pdf");
    render_character(&character, root.join("assets/character-sheet.pdf"), &output)
        .expect("production render");

    let values = read_field_values(
        &output,
        ["Text7", "Text8", "Text13"].into_iter().map(str::to_owned),
    )
    .expect("read production fields");
    for (field, expected) in [("Text7", "Rogue"), ("Text8", "Tiefling"), ("Text13", "14")] {
        assert_eq!(
            values[field].as_str().expect("text value"),
            expected.as_bytes(),
            "{field}"
        );
    }

    let attack_appearance = appearance_content(&output, "Text33");
    let font_size: f32 = attack_appearance
        .split("/Helv ")
        .nth(1)
        .and_then(|value| value.split(" Tf").next())
        .expect("font size")
        .parse()
        .expect("numeric font size");
    assert!((0.0..12.0).contains(&font_size));
    assert!(attack_appearance.contains(" Tj"));
    assert!(appearance_content(&output, "Text54").matches(" Td").count() > 2);
    fs::remove_dir_all(directory).expect("remove temporary proof directory");
}
