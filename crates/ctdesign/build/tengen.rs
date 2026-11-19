use serde_json::Value;

pub(super) const THEMES: [(&str, &str); 3] = [
    ("LIGHT", "light"),
    ("DARK", "dark"),
    ("DARK_TINT", "dark-tint"),
];

#[derive(Clone, Copy)]
pub(super) struct Look<'a> {
    pub(super) theme: &'a str,
    pub(super) opaque: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Color {
    r: u8,
    g: u8,
    b: u8,
    pub(super) a: u8,
}

pub(super) struct Tengen {
    pub(super) root: Value,
    pub(super) structs: String,
}

impl Tengen {
    pub(super) fn find(&self, path: &str) -> Option<&Value> {
        path.split('.')
            .try_fold(&self.root, |value, part| value.get(part))
    }

    pub(super) fn get(&self, path: &str) -> &Value {
        self.find(path)
            .unwrap_or_else(|| panic!("ctdesign: missing `{path}` in tengen"))
    }

    pub(super) fn string(&self, path: &str) -> &str {
        self.get(path)
            .as_str()
            .unwrap_or_else(|| panic!("ctdesign: `{path}` in tengen is not a string"))
    }

    pub(super) fn number(&self, path: &str) -> f64 {
        let value = self.get(path);
        value.as_f64().unwrap_or_else(|| match value.as_str() {
            Some(reference) => self.number(reference),
            None => panic!("ctdesign: `{path}` in tengen is neither a number nor a reference"),
        })
    }

    pub(super) fn color(&self, path: &str, look: Look) -> Color {
        let text = self.string(path);
        let name = text
            .strip_prefix('{')
            .and_then(|name| name.strip_suffix('}'))
            .unwrap_or(text);
        let themed = format!("themes.{}.{name}", look.theme);
        let color = match self.find(&themed) {
            Some(_) => parse_color(self.string(&themed)),
            None => parse_color(text),
        };
        if look.opaque {
            let bg = parse_color(self.string(&format!("themes.{}.bg-window", look.theme)));
            opaque_over(color, bg)
        } else {
            color
        }
    }

    pub(super) fn font(&self, style: &str) -> String {
        let family = families(self.string(&format!(
            "font.families.{}",
            self.string(&format!("{style}.family"))
        )))
        .next()
        .unwrap_or_default();
        let int = |key: &str| self.number(&format!("{style}.{key}")) as i64;
        format!(
            "Font {{ family: {family:?}, size: {}, line_height: {}, weight: {} }}",
            int("size"),
            int("line-height"),
            int("weight")
        )
    }

    pub(super) fn literal(&self, ty: &str, path: &str, overrides: &[&str], look: Look) -> String {
        let pair = |n: &[String]| format!("({}, {})", n[0], n[1]);
        match ty {
            "f32" | "f64" => float(self.number(path)),
            "u16" | "u32" | "u64" | "i32" | "usize" => (self.number(path) as i64).to_string(),
            "Rgba" => rgba(self.color(path, look)),
            "Font" => match self.get(path).as_str() {
                Some(style) => self.font(&format!("font.styles.{style}")),
                None => self.font(path),
            },
            "&'static [(f32, f32)]" => self.list(path, pair),
            "&'static [Segment]" => self.list(path, |n| {
                format!("Segment {{ a: {}, b: {} }}", pair(&n[..2]), pair(&n[2..]))
            }),
            "&'static [Triangle]" => self.list(path, |n| {
                format!(
                    "Triangle([{}, {}, {}])",
                    pair(&n[..2]),
                    pair(&n[2..4]),
                    pair(&n[4..])
                )
            }),
            ty if ty.starts_with("[f32;") => {
                format!("[{}]", self.floats(self.get(path)).join(", "))
            }
            ty if ty.starts_with("[Rgba;") => {
                let colors: Vec<String> = self
                    .get(path)
                    .as_array()
                    .unwrap_or_else(|| panic!("ctdesign: `{path}` in tengen is not a list"))
                    .iter()
                    .map(|color| rgba(parse_color(color.as_str().expect("ctdesign: colours"))))
                    .collect();
                format!("[{}]", colors.join(", "))
            }
            ty => self.structure(ty, path, overrides, look),
        }
    }

    fn structure(&self, ty: &str, prefix: &str, overrides: &[&str], look: Look) -> String {
        let fields: Vec<String> = self
            .fields(ty)
            .map(|(field, field_ty)| {
                let target = overrides
                    .iter()
                    .find_map(|text| text.strip_prefix(field)?.strip_prefix('='));
                let value = match target {
                    Some(text) if text.starts_with('=') => text[1..].to_string(),
                    Some(path) => {
                        let relative = format!("{prefix}.{path}");
                        let path = if self.find(&relative).is_some() {
                            &relative
                        } else {
                            path
                        };
                        self.literal(field_ty, path, &[], look)
                    }
                    None => self.literal(field_ty, &self.key(prefix, field), &[], look),
                };
                format!("{field}: {value}")
            })
            .collect();
        format!("{ty} {{ {} }}", fields.join(", "))
    }

    pub(super) fn constant(
        &self,
        name: &str,
        ty: &str,
        path: &str,
        overrides: &[&str],
        look: Look,
    ) -> String {
        let value = self.literal(ty, path, overrides, look);
        format!("pub const {name}: {ty} = {value};\n\n")
    }

    fn fields(&self, ty: &str) -> impl Iterator<Item = (&str, &str)> {
        let start = self
            .structs
            .find(&format!("pub struct {ty} {{\n"))
            .unwrap_or_else(|| panic!("ctdesign: no struct `{ty}` in src/lib.rs"));
        self.structs[start..]
            .lines()
            .skip(1)
            .take_while(|line| *line != "}")
            .filter_map(|line| {
                line.trim()
                    .strip_prefix("pub ")?
                    .strip_suffix(',')?
                    .split_once(": ")
            })
    }

    fn key(&self, prefix: &str, field: &str) -> String {
        self.nested_key(prefix, field)
            .unwrap_or_else(|| panic!("ctdesign: no key for `{field}` under `{prefix}`"))
    }

    fn nested_key(&self, prefix: &str, field: &str) -> Option<String> {
        let flat = format!("{prefix}.{}", field.replace('_', "-"));
        if self.find(&flat).is_some() {
            return Some(flat);
        }
        field.match_indices('_').find_map(|(split, _)| {
            let group = format!("{prefix}.{}", field[..split].replace('_', "-"));
            self.find(&group)?.as_object()?;
            self.nested_key(&group, &field[split + 1..])
        })
    }

    fn floats(&self, value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap_or_else(|| panic!("ctdesign: `{value}` in tengen is not a list"))
            .iter()
            .map(|n| float(n.as_f64().expect("ctdesign: list of numbers")))
            .collect()
    }

    fn list(&self, path: &str, item: impl Fn(&[String]) -> String) -> String {
        let items: Vec<String> = self
            .get(path)
            .as_array()
            .unwrap_or_else(|| panic!("ctdesign: `{path}` in tengen is not a list"))
            .iter()
            .map(|numbers| item(&self.floats(numbers)))
            .collect();
        format!("&[{}]", items.join(", "))
    }
}

pub(super) fn families(stack: &str) -> impl Iterator<Item = &str> {
    stack
        .split(',')
        .map(|family| family.trim().trim_matches('"'))
}

pub(super) fn float(value: f64) -> String {
    let text = value.to_string();
    if text.contains(['.', 'e', 'E']) {
        text
    } else {
        format!("{text}.0")
    }
}

pub(super) fn parse_color(text: &str) -> Color {
    let channel = |index: usize| {
        text.strip_prefix('#')
            .and_then(|hex| hex.get(index..index + 2))
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
            .unwrap_or_else(|| panic!("ctdesign: invalid colour `{text}` in tengen"))
    };
    let a = match text.len() {
        7 => 255,
        9 => channel(6),
        _ => panic!("ctdesign: invalid colour `{text}` in tengen"),
    };
    Color {
        r: channel(0),
        g: channel(2),
        b: channel(4),
        a,
    }
}

pub(super) fn rgba(color: Color) -> String {
    let Color { r, g, b, a } = color;
    if a == 255 {
        format!("Rgba::rgb8(0x{r:02x}, 0x{g:02x}, 0x{b:02x})")
    } else {
        format!(
            "Rgba::rgba8(0x{r:02x}, 0x{g:02x}, 0x{b:02x}, {:.6})",
            f64::from(a) / 255.0
        )
    }
}

fn opaque_over(color: Color, bg: Color) -> Color {
    let alpha = f32::from(color.a) / 255.0;
    let blend =
        |fg: u8, back: u8| (f32::from(fg) * alpha + f32::from(back) * (1.0 - alpha)).round() as u8;
    Color {
        r: blend(color.r, bg.r),
        g: blend(color.g, bg.g),
        b: blend(color.b, bg.b),
        a: 255,
    }
}
