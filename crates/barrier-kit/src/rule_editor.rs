use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Size};
use vow_design::{SPACE_1, SPACE_2, TEXT};

use crate::compact::IconButton;
use crate::controls::TextField;
use crate::i18n::text;
use crate::popup_button::PopUpButton;
use crate::style::{Palette, text};
use crate::symbol::Symbol;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuleKind {
    Text,
    Number,
    Choice(&'static [&'static str]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuleField {
    pub label: &'static str,
    pub kind: RuleKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Contains,
    BeginsWith,
    EndsWith,
    Is,
    IsNot,
    Greater,
    Less,
}

pub fn operators(kind: RuleKind) -> &'static [Operator] {
    match kind {
        RuleKind::Text => &[
            Operator::Contains,
            Operator::BeginsWith,
            Operator::EndsWith,
            Operator::Is,
            Operator::IsNot,
        ],
        RuleKind::Number => &[
            Operator::Is,
            Operator::IsNot,
            Operator::Greater,
            Operator::Less,
        ],
        RuleKind::Choice(_) => &[Operator::Is, Operator::IsNot],
    }
}

fn operator_label(operator: Operator) -> &'static str {
    match operator {
        Operator::Contains => text!("rule-contains"),
        Operator::BeginsWith => text!("rule-begins-with"),
        Operator::EndsWith => text!("rule-ends-with"),
        Operator::Is => text!("rule-is"),
        Operator::IsNot => text!("rule-is-not"),
        Operator::Greater => text!("rule-greater"),
        Operator::Less => text!("rule-less"),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub id: u64,
    pub field: usize,
    pub operator: Operator,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Predicate {
    pub all: bool,
    pub rules: Vec<Rule>,
}

impl Rule {
    pub fn new(id: u64, fields: &[RuleField], field: usize) -> Self {
        let kind = fields[field].kind;
        Self {
            id,
            field,
            operator: operators(kind)[0],
            value: match kind {
                RuleKind::Choice(choices) => choices[0].to_owned(),
                _ => String::new(),
            },
        }
    }

    pub fn matches(&self, kind: RuleKind, actual: &str) -> bool {
        if kind == RuleKind::Number {
            let (Ok(actual), Ok(wanted)) =
                (actual.trim().parse::<f64>(), self.value.trim().parse())
            else {
                return false;
            };
            return match self.operator {
                Operator::Is => actual == wanted,
                Operator::IsNot => actual != wanted,
                Operator::Greater => actual > wanted,
                Operator::Less => actual < wanted,
                _ => false,
            };
        }
        let (actual, wanted) = (actual.to_lowercase(), self.value.to_lowercase());
        match self.operator {
            Operator::Contains => actual.contains(&wanted),
            Operator::BeginsWith => actual.starts_with(&wanted),
            Operator::EndsWith => actual.ends_with(&wanted),
            Operator::Is => actual == wanted,
            Operator::IsNot => actual != wanted,
            Operator::Greater | Operator::Less => false,
        }
    }
}

impl Predicate {
    pub fn evaluate(&self, fields: &[RuleField], value_of: impl Fn(usize) -> String) -> bool {
        let mut results = self
            .rules
            .iter()
            .map(|rule| rule.matches(fields[rule.field].kind, &value_of(rule.field)));
        if self.all {
            results.all(|matched| matched)
        } else {
            self.rules.is_empty() || results.any(|matched| matched)
        }
    }

    fn update(&mut self, id: u64, change: impl FnOnce(&mut Rule)) {
        if let Some(rule) = self.rules.iter_mut().find(|rule| rule.id == id) {
            change(rule);
        }
    }

    fn insert_after(&mut self, id: u64, fields: &[RuleField]) {
        let next = self.rules.iter().map(|rule| rule.id).max().unwrap_or(0) + 1;
        let at = self
            .rules
            .iter()
            .position(|rule| rule.id == id)
            .map_or(self.rules.len(), |index| index + 1);
        self.rules.insert(at, Rule::new(next, fields, 0));
    }
}

#[derive(PartialEq)]
pub struct RuleEditor {
    label: &'static str,
    fields: Vec<RuleField>,
    predicate: State<Predicate>,
}

impl RuleEditor {
    pub fn new(label: &'static str, fields: Vec<RuleField>, predicate: State<Predicate>) -> Self {
        Self {
            label,
            fields,
            predicate,
        }
    }
}

impl Component for RuleEditor {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut predicate = self.predicate;
        let current = predicate.read().clone();
        let header = rect()
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SPACE_2)
            .child(
                PopUpButton::new(text!("rule-match"))
                    .item(text!("rule-all"))
                    .item(text!("rule-any"))
                    .selected(usize::from(!current.all))
                    .on_change(move |index: usize| predicate.write().all = index == 0),
            )
            .child(text(text!("rule-of-the-following"), TEXT, palette.label))
            .maybe_child(current.rules.is_empty().then(|| {
                let fields = self.fields.clone();
                IconButton::new(Symbol::Plus, text!("rule-add"))
                    .on_press(move |_| predicate.write().insert_after(0, &fields))
            }));
        let single = current.rules.len() == 1;
        let rows = current.rules.iter().map(|rule| {
            RuleRow {
                id: rule.id,
                fields: self.fields.clone(),
                predicate,
                removable: !single,
            }
            .into_element()
        });
        rect()
            .a11y_role(AccessibilityRole::Group)
            .a11y_alt(self.label)
            .width(Size::fill())
            .direction(Direction::Vertical)
            .spacing(SPACE_2)
            .child(header)
            .children(rows)
    }
}

#[derive(PartialEq)]
struct RuleRow {
    id: u64,
    fields: Vec<RuleField>,
    predicate: State<Predicate>,
    removable: bool,
}

impl Component for RuleRow {
    fn render(&self) -> impl IntoElement {
        let id = self.id;
        let mut predicate = self.predicate;
        let Some(rule) = predicate
            .read()
            .rules
            .iter()
            .find(|rule| rule.id == id)
            .cloned()
        else {
            return rect().into_element();
        };
        let kind = self.fields[rule.field].kind;
        let shown = rule.value.clone();
        let mut value = use_state(move || shown);
        if *value.peek() != rule.value {
            value.set(rule.value.clone());
        }
        use_side_effect(move || {
            let typed = value.read().clone();
            if predicate
                .peek()
                .rules
                .iter()
                .any(|rule| rule.id == id && rule.value != typed)
            {
                predicate.write().update(id, |rule| rule.value = typed);
            }
        });

        let fields = self.fields.clone();
        let field_picker = self
            .fields
            .iter()
            .fold(PopUpButton::new(text!("rule-field")), |picker, field| {
                picker.item(field.label)
            })
            .selected(rule.field)
            .on_change(move |index: usize| {
                let fresh = Rule::new(id, &fields, index);
                predicate.write().update(id, |rule| *rule = fresh);
            });
        let operator_picker = operators(kind)
            .iter()
            .fold(
                PopUpButton::new(text!("rule-operator")),
                |picker, operator| picker.item(operator_label(*operator)),
            )
            .selected(
                operators(kind)
                    .iter()
                    .position(|operator| *operator == rule.operator)
                    .unwrap_or(0),
            )
            .on_change(move |index: usize| {
                predicate
                    .write()
                    .update(id, |rule| rule.operator = operators(kind)[index]);
            });
        let editor = match kind {
            RuleKind::Choice(choices) => choices
                .iter()
                .fold(PopUpButton::new(text!("rule-value")), |picker, choice| {
                    picker.item(choice)
                })
                .selected(choices.iter().position(|c| *c == rule.value).unwrap_or(0))
                .on_change(move |index: usize| {
                    predicate
                        .write()
                        .update(id, |rule| rule.value = choices[index].to_owned());
                })
                .into_element(),
            RuleKind::Text | RuleKind::Number => {
                TextField::new(value, text!("rule-value")).into_element()
            }
        };
        let fields = self.fields.clone();
        rect()
            .width(Size::fill())
            .direction(Direction::Horizontal)
            .content(torin::prelude::Content::flex())
            .cross_align(Alignment::Center)
            .spacing(SPACE_1)
            .child(field_picker)
            .child(operator_picker)
            .child(rect().width(Size::flex(1.0)).child(editor))
            .child(
                IconButton::new(Symbol::Minus, text!("rule-remove"))
                    .disabled(!self.removable)
                    .on_press(move |_| predicate.write().rules.retain(|rule| rule.id != id)),
            )
            .child(
                IconButton::new(Symbol::Plus, text!("rule-add"))
                    .on_press(move |_| predicate.write().insert_after(id, &fields)),
            )
            .into_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELDS: [RuleField; 2] = [
        RuleField {
            label: "Name",
            kind: RuleKind::Text,
        },
        RuleField {
            label: "Size",
            kind: RuleKind::Number,
        },
    ];

    fn rule(field: usize, operator: Operator, value: &str) -> Rule {
        Rule {
            id: 0,
            field,
            operator,
            value: value.into(),
        }
    }

    #[test]
    fn text_rules_ignore_case_and_number_rules_compare_parsed_values() {
        let name = |_| "Informe.PDF".to_string();
        assert!(rule(0, Operator::EndsWith, ".pdf").matches(RuleKind::Text, &name(0)));
        assert!(rule(1, Operator::Greater, "9").matches(RuleKind::Number, "10"));
        assert!(!rule(1, Operator::Greater, "9").matches(RuleKind::Number, "abc"));
    }

    #[test]
    fn all_needs_every_rule_and_any_needs_one() {
        let value = |field: usize| ["notas", "3"][field].to_string();
        let mut predicate = Predicate {
            all: true,
            rules: vec![
                rule(0, Operator::Contains, "not"),
                rule(1, Operator::Greater, "5"),
            ],
        };
        assert!(!predicate.evaluate(&FIELDS, value));
        predicate.all = false;
        assert!(predicate.evaluate(&FIELDS, value));
    }

    #[test]
    fn a_new_rule_goes_after_the_pressed_one_with_a_fresh_id() {
        let mut predicate = Predicate {
            all: true,
            rules: vec![Rule::new(1, &FIELDS, 0), Rule::new(2, &FIELDS, 1)],
        };
        predicate.insert_after(1, &FIELDS);
        let ids: Vec<u64> = predicate.rules.iter().map(|rule| rule.id).collect();
        assert_eq!(ids, [1, 3, 2]);
    }
}
