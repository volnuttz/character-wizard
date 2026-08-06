//! Text appearance-stream generation and layout.

use lopdf::{
    Document, Object, ObjectId, Stream,
    content::{Content, Operation},
    dictionary,
};

type Result<T> = std::result::Result<T, String>;

pub(super) fn text_appearance(
    document: &mut Document,
    widget: ObjectId,
    target: ObjectId,
    value: &str,
) -> Result<ObjectId> {
    let widget_dictionary = document
        .get_object(widget)
        .and_then(Object::as_dict)
        .map_err(|error| error.to_string())?;
    let rectangle = widget_dictionary
        .get(b"Rect")
        .and_then(Object::as_array)
        .map_err(|error| error.to_string())?;
    if rectangle.len() != 4 {
        return Err("text widget rectangle must have four coordinates".to_owned());
    }
    let width = (rectangle[2].as_f32().map_err(|error| error.to_string())?
        - rectangle[0].as_f32().map_err(|error| error.to_string())?)
    .abs();
    let height = (rectangle[3].as_f32().map_err(|error| error.to_string())?
        - rectangle[1].as_f32().map_err(|error| error.to_string())?)
    .abs();
    let flags = document
        .get_object(target)
        .and_then(Object::as_dict)
        .ok()
        .and_then(|field| field.get(b"Ff").ok())
        .or_else(|| widget_dictionary.get(b"Ff").ok())
        .and_then(|flags| flags.as_i64().ok())
        .unwrap_or(0);
    let multiline = flags & 4096 != 0 || value.contains('\n');
    let (font_size, lines) = text_layout(value, width - 4.0, height - 2.0, multiline);
    let leading = font_size * 1.156;
    let initial_y = if multiline {
        height - font_size * 0.85
    } else {
        (height - font_size) * 0.5 + font_size * 0.18
    };
    let mut operations = vec![
        Operation::new("q", vec![]),
        Operation::new("BMC", vec![Object::Name(b"Tx".to_vec())]),
        Operation::new("q", vec![]),
        Operation::new(
            "re",
            vec![
                2.into(),
                1.into(),
                (width - 4.0).into(),
                (height - 2.0).into(),
            ],
        ),
        Operation::new("W", vec![]),
        Operation::new("n", vec![]),
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec![Object::Name(b"Helv".to_vec()), font_size.into()]),
        Operation::new("g", vec![0.into()]),
        Operation::new("Td", vec![2.into(), initial_y.into()]),
    ];
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            operations.push(Operation::new("Td", vec![0.into(), (-leading).into()]));
        }
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(appearance_text(line))],
        ));
    }
    operations.extend([
        Operation::new("ET", vec![]),
        Operation::new("Q", vec![]),
        Operation::new("EMC", vec![]),
        Operation::new("Q", vec![]),
    ]);
    let resources = Object::Dictionary(dictionary! {
        "Font" => Object::Dictionary(dictionary! { "Helv" => helvetica_resource(document)? })
    });
    let stream = Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Form",
            "BBox" => vec![0.into(), 0.into(), width.into(), height.into()],
            "Resources" => resources,
        },
        Content { operations }
            .encode()
            .map_err(|error| error.to_string())?,
    );
    Ok(document.add_object(stream))
}

fn helvetica_resource(document: &Document) -> Result<Object> {
    let (_, catalog) = document
        .dereference(
            document
                .trailer
                .get(b"Root")
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let catalog = catalog.as_dict().map_err(|error| error.to_string())?;
    let (_, form) = document
        .dereference(
            catalog
                .get(b"AcroForm")
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    let form = form.as_dict().map_err(|error| error.to_string())?;
    let (_, resources) = document
        .dereference(form.get(b"DR").map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    let resources = resources.as_dict().map_err(|error| error.to_string())?;
    let (_, fonts) = document
        .dereference(resources.get(b"Font").map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    fonts
        .as_dict()
        .map_err(|error| error.to_string())?
        .get(b"Helv")
        .cloned()
        .map_err(|error| error.to_string())
}

fn text_layout(value: &str, width: f32, height: f32, multiline: bool) -> (f32, Vec<String>) {
    let normalized = if multiline {
        value.to_owned()
    } else {
        value.replace(['\r', '\n'], " ")
    };
    for half_points in (6_u8..=24).rev() {
        let size = f32::from(half_points) / 2.0;
        let lines = if multiline {
            wrap_text(&normalized, width, size)
        } else {
            vec![normalized.clone()]
        };
        let fits_width = lines
            .iter()
            .all(|line| estimated_text_width(line, size) <= width);
        let fits_height = if multiline {
            f32::from(u16::try_from(lines.len()).unwrap_or(u16::MAX)) * size * 1.156 <= height
        } else {
            size * 1.156 <= height
        };
        if fits_width && fits_height {
            return (size, lines);
        }
    }
    let size = 3.0;
    let lines = if multiline {
        wrap_text(&normalized, width, size)
    } else {
        vec![normalized]
    };
    (size, lines)
}

fn wrap_text(value: &str, width: f32, font_size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in value.replace('\r', "").split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if line.is_empty() || estimated_text_width(&candidate, font_size) <= width {
                line = candidate;
            } else {
                lines.push(std::mem::take(&mut line));
                word.clone_into(&mut line);
            }
        }
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn estimated_text_width(value: &str, font_size: f32) -> f32 {
    value
        .chars()
        .map(|character| match character {
            'i' | 'l' | 'I' | '!' | '.' | ',' | ':' | ';' | '\'' => 0.25,
            'm' | 'w' | 'M' | 'W' | '@' => 0.85,
            ' ' => 0.28,
            _ => 0.52,
        })
        .sum::<f32>()
        * font_size
}

fn appearance_text(value: &str) -> String {
    value
        .chars()
        .map(
            |character| {
                if character.is_ascii() { character } else { '?' }
            },
        )
        .collect()
}
