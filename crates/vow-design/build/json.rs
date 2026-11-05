use super::*;

pub(super) fn load(path: &Path) -> Value {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("vow-design: cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("vow-design: invalid JSON in {}: {error}", path.display()))
}

pub(super) fn value<'a>(root: &'a Value, path: &str, file: &str) -> &'a Value {
    let mut current = root;
    for part in path.split('.') {
        current = current.get(part).unwrap_or_else(|| {
            panic!("vow-design: missing field `{path}` in {file} (needed by vow-design)")
        });
    }
    current
}

pub(super) fn number(root: &Value, path: &str, file: &str) -> f64 {
    value(root, path, file)
        .as_f64()
        .unwrap_or_else(|| panic!("vow-design: `{path}` in {file} is not a number"))
}

pub(super) fn integer(root: &Value, path: &str, file: &str) -> i64 {
    value(root, path, file)
        .as_i64()
        .unwrap_or_else(|| panic!("vow-design: `{path}` in {file} is not an integer"))
}

pub(super) fn string<'a>(root: &'a Value, path: &str, file: &str) -> &'a str {
    value(root, path, file)
        .as_str()
        .unwrap_or_else(|| panic!("vow-design: `{path}` in {file} is not a string"))
}

pub(super) fn metric(tokens: &Value, components: &Value, path: &str, file: &str) -> f64 {
    let value = value(components, path, file);
    if let Some(number) = value.as_f64() {
        return number;
    }
    let reference = value.as_str().unwrap_or_else(|| {
        panic!("vow-design: `{path}` in {file} is neither a number nor a token reference")
    });
    let (group, key) = reference.split_once('.').unwrap_or_else(|| {
        panic!("vow-design: `{path}` in {file} has an invalid reference `{reference}`")
    });
    number(tokens, &format!("{group}.{key}"), TOKENS)
}

pub(super) fn float(value: f64) -> String {
    let text = value.to_string();
    if text.contains('.') || text.contains('e') || text.contains('E') {
        text
    } else {
        format!("{text}.0")
    }
}

pub(super) fn first_family(families: &str) -> String {
    families
        .split(',')
        .next()
        .unwrap_or(families)
        .trim()
        .trim_matches('"')
        .to_string()
}

pub(super) fn font(tokens: &Value, style: &str) -> String {
    let path = format!("font.styles.{style}");
    let family_key = string(tokens, &format!("{path}.family"), TOKENS);
    let families = string(tokens, &format!("font.families.{family_key}"), TOKENS);
    let family = first_family(families);
    let size = integer(tokens, &format!("{path}.size"), TOKENS);
    let line_height = integer(tokens, &format!("{path}.line-height"), TOKENS);
    let weight = integer(tokens, &format!("{path}.weight"), TOKENS);
    format!(
        "Font {{ family: {family:?}, size: {size}, line_height: {line_height}, weight: {weight} }}"
    )
}

pub(super) fn pub_const(out: &mut String, name: &str) {
    out.push_str(&format!("pub const {name}"));
}

pub(super) fn floats(components: &Value, path: &str) -> String {
    value(components, path, COMPONENTS)
        .as_array()
        .unwrap_or_else(|| panic!("vow-design: `{path}` must be a list"))
        .iter()
        .map(|v| float(v.as_f64().expect("number")))
        .collect::<Vec<_>>()
        .join(", ")
}
