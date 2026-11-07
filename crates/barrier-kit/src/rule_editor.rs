use ct_design::{BODY, DISCLOSURE, SPACE_1, SPACE_2};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Direction, Gaps, Size};

use crate::compact::IconButton;
use crate::controls::TextField;
use crate::i18n::text;
use crate::interaction::{track_modifiers, use_held_modifiers};
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
pub enum Criterion {
    Rule(Rule),
    Group { id: u64, predicate: Predicate },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Predicate {
    pub all: bool,
    pub rules: Vec<Criterion>,
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

impl Criterion {
    fn id(&self) -> u64 {
        match self {
            Criterion::Rule(rule) => rule.id,
            Criterion::Group { id, .. } => *id,
        }
    }
}

impl Predicate {
    pub fn evaluate(&self, fields: &[RuleField], value_of: &impl Fn(usize) -> String) -> bool {
        let mut results = self.rules.iter().map(|criterion| match criterion {
            Criterion::Rule(rule) => rule.matches(fields[rule.field].kind, &value_of(rule.field)),
            Criterion::Group { predicate, .. } => predicate.evaluate(fields, value_of),
        });
        if self.all {
            results.all(|matched| matched)
        } else {
            self.rules.is_empty() || results.any(|matched| matched)
        }
    }

    fn max_id(&self) -> u64 {
        self.rules
            .iter()
            .map(|criterion| match criterion {
                Criterion::Rule(rule) => rule.id,
                Criterion::Group { id, predicate } => (*id).max(predicate.max_id()),
            })
            .max()
            .unwrap_or(0)
    }

    pub fn rule(&self, id: u64) -> Option<&Rule> {
        self.rules.iter().find_map(|criterion| match criterion {
            Criterion::Rule(rule) => (rule.id == id).then_some(rule),
            Criterion::Group { predicate, .. } => predicate.rule(id),
        })
    }

    fn group_mut(&mut self, id: u64) -> Option<&mut Predicate> {
        self.rules.iter_mut().find_map(|criterion| match criterion {
            Criterion::Group {
                id: group,
                predicate,
            } => {
                if *group == id {
                    Some(predicate)
                } else {
                    predicate.group_mut(id)
                }
            }
            Criterion::Rule(_) => None,
        })
    }

    fn rule_mut(&mut self, id: u64) -> Option<&mut Rule> {
        self.rules.iter_mut().find_map(|criterion| match criterion {
            Criterion::Rule(rule) => (rule.id == id).then_some(rule),
            Criterion::Group { predicate, .. } => predicate.rule_mut(id),
        })
    }

    fn update(&mut self, id: u64, change: impl FnOnce(&mut Rule)) {
        if let Some(rule) = self.rule_mut(id) {
            change(rule);
        }
    }

    fn set_all(&mut self, id: u64, all: bool) {
        if let Some(group) = self.group_mut(id) {
            group.all = all;
        }
    }

    pub fn remove(&mut self, id: u64) {
        self.rules.retain(|criterion| criterion.id() != id);
        for criterion in &mut self.rules {
            if let Criterion::Group { predicate, .. } = criterion {
                predicate.remove(id);
            }
        }
        self.rules.retain(|criterion| {
            !matches!(criterion, Criterion::Group { predicate, .. } if predicate.rules.is_empty())
        });
    }

    pub fn insert_after(&mut self, id: u64, fields: &[RuleField], compound: bool) {
        let next = self.max_id() + 1;
        let fresh = if compound {
            Criterion::Group {
                id: next,
                predicate: Predicate {
                    all: true,
                    rules: vec![Criterion::Rule(Rule::new(next + 1, fields, 0))],
                },
            }
        } else {
            Criterion::Rule(Rule::new(next, fields, 0))
        };
        let mut slot = Some(fresh);
        if !self.place_after(id, &mut slot) {
            self.rules.extend(slot);
        }
    }

    fn place_after(&mut self, id: u64, fresh: &mut Option<Criterion>) -> bool {
        if let Some(index) = self.rules.iter().position(|criterion| criterion.id() == id) {
            if let Some(fresh) = fresh.take() {
                self.rules.insert(index + 1, fresh);
            }
            return true;
        }
        self.rules.iter_mut().any(|criterion| match criterion {
            Criterion::Group { predicate, .. } => predicate.place_after(id, fresh),
            Criterion::Rule(_) => false,
        })
    }

    fn rows(&self, depth: usize, out: &mut Vec<(usize, Criterion)>) {
        for criterion in &self.rules {
            out.push((depth, criterion.clone()));
            if let Criterion::Group { predicate, .. } = criterion {
                predicate.rows(depth + 1, out);
            }
        }
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

fn add_button(
    id: u64,
    fields: Vec<RuleField>,
    mut predicate: State<Predicate>,
    compound: bool,
) -> IconButton {
    let (symbol, label) = if compound {
        (Symbol::Ellipsis, text!("rule-add-compound"))
    } else {
        (Symbol::Plus, text!("rule-add"))
    };
    IconButton::new(symbol, label)
        .on_press(move |_| predicate.write().insert_after(id, &fields, compound))
}

fn match_picker(all: bool, on_change: impl FnMut(usize) + 'static) -> impl IntoElement {
    let palette = Palette::current();
    rect()
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .spacing(SPACE_2)
        .child(
            PopUpButton::new(text!("rule-match"))
                .item(text!("rule-all"))
                .item(text!("rule-any"))
                .selected(usize::from(!all))
                .on_change(on_change),
        )
        .child(text(text!("rule-of-the-following"), BODY, palette.label))
}

fn indented(depth: usize, row: Rect) -> Rect {
    row.width(Size::fill())
        .padding(Gaps::new(0.0, 0.0, 0.0, depth as f32 * DISCLOSURE.indent))
}

impl Component for RuleEditor {
    fn render(&self) -> impl IntoElement {
        let mut predicate = self.predicate;
        let held = use_held_modifiers();
        let compound = held.read().contains(Modifiers::ALT);
        let current = predicate.read().clone();
        let header = rect()
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SPACE_2)
            .child(match_picker(current.all, move |index: usize| {
                predicate.write().all = index == 0
            }))
            .maybe_child(
                current
                    .rules
                    .is_empty()
                    .then(|| add_button(0, self.fields.clone(), predicate, compound)),
            );
        let single = current.rules.len() == 1;
        let mut rows = Vec::new();
        current.rows(0, &mut rows);
        let rows = rows.into_iter().map(|(depth, criterion)| match criterion {
            Criterion::Rule(rule) => RuleRow {
                id: rule.id,
                fields: self.fields.clone(),
                predicate,
                removable: !(single && depth == 0),
                depth,
                compound,
            }
            .into_element(),
            Criterion::Group {
                id,
                predicate: group,
            } => {
                let mut editor = predicate;
                indented(
                    depth,
                    rect()
                        .direction(Direction::Horizontal)
                        .content(torin::prelude::Content::flex())
                        .cross_align(Alignment::Center)
                        .spacing(SPACE_1),
                )
                .child(match_picker(group.all, move |index: usize| {
                    editor.write().set_all(id, index == 0)
                }))
                .child(rect().width(Size::flex(1.0)))
                .child(
                    IconButton::new(Symbol::Minus, text!("rule-remove"))
                        .disabled(single && depth == 0)
                        .on_press(move |_| editor.write().remove(id)),
                )
                .child(add_button(id, self.fields.clone(), predicate, compound))
                .into_element()
            }
        });
        track_modifiers(
            rect()
                .a11y_role(AccessibilityRole::Group)
                .a11y_alt(self.label)
                .width(Size::fill())
                .direction(Direction::Vertical)
                .spacing(SPACE_2)
                .child(header)
                .children(rows),
            held,
        )
    }
}

#[derive(PartialEq)]
struct RuleRow {
    id: u64,
    fields: Vec<RuleField>,
    predicate: State<Predicate>,
    removable: bool,
    depth: usize,
    compound: bool,
}

impl Component for RuleRow {
    fn render(&self) -> impl IntoElement {
        let id = self.id;
        let mut predicate = self.predicate;
        let Some(rule) = predicate.read().rule(id).cloned() else {
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
                .rule(id)
                .is_some_and(|rule| rule.value != typed)
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
        indented(
            self.depth,
            rect()
                .direction(Direction::Horizontal)
                .content(torin::prelude::Content::flex())
                .cross_align(Alignment::Center)
                .spacing(SPACE_1),
        )
        .child(field_picker)
        .child(operator_picker)
        .child(rect().width(Size::flex(1.0)).child(editor))
        .child(
            IconButton::new(Symbol::Minus, text!("rule-remove"))
                .disabled(!self.removable)
                .on_press(move |_| predicate.write().remove(id)),
        )
        .child(add_button(
            id,
            self.fields.clone(),
            predicate,
            self.compound,
        ))
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

    fn rule(id: u64, field: usize, operator: Operator, value: &str) -> Criterion {
        Criterion::Rule(Rule {
            id,
            field,
            operator,
            value: value.into(),
        })
    }

    fn ids(predicate: &Predicate) -> Vec<(usize, u64)> {
        let mut rows = Vec::new();
        predicate.rows(0, &mut rows);
        rows.iter()
            .map(|(depth, criterion)| (*depth, criterion.id()))
            .collect()
    }

    #[test]
    fn text_rules_ignore_case_and_number_rules_compare_parsed_values() {
        let check = |criterion: Criterion, kind, actual: &str| match criterion {
            Criterion::Rule(rule) => rule.matches(kind, actual),
            Criterion::Group { .. } => unreachable!(),
        };
        assert!(check(
            rule(0, 0, Operator::EndsWith, ".pdf"),
            RuleKind::Text,
            "Informe.PDF"
        ));
        assert!(check(
            rule(0, 1, Operator::Greater, "9"),
            RuleKind::Number,
            "10"
        ));
        assert!(!check(
            rule(0, 1, Operator::Greater, "9"),
            RuleKind::Number,
            "abc"
        ));
    }

    #[test]
    fn all_needs_every_rule_and_any_needs_one() {
        let value = |field: usize| ["notas", "3"][field].to_string();
        let mut predicate = Predicate {
            all: true,
            rules: vec![
                rule(1, 0, Operator::Contains, "not"),
                rule(2, 1, Operator::Greater, "5"),
            ],
        };
        assert!(!predicate.evaluate(&FIELDS, &value));
        predicate.all = false;
        assert!(predicate.evaluate(&FIELDS, &value));
    }

    #[test]
    fn a_nested_group_is_evaluated_as_one_criterion() {
        let value = |field: usize| ["notas", "3"][field].to_string();
        let predicate = Predicate {
            all: true,
            rules: vec![
                rule(1, 0, Operator::Contains, "not"),
                Criterion::Group {
                    id: 2,
                    predicate: Predicate {
                        all: false,
                        rules: vec![
                            rule(3, 1, Operator::Greater, "5"),
                            rule(4, 1, Operator::Less, "4"),
                        ],
                    },
                },
            ],
        };
        assert!(predicate.evaluate(&FIELDS, &value));
    }

    #[test]
    fn a_new_rule_goes_after_the_pressed_one_with_a_fresh_id() {
        let mut predicate = Predicate {
            all: true,
            rules: vec![rule(1, 0, Operator::Is, ""), rule(2, 1, Operator::Is, "")],
        };
        predicate.insert_after(1, &FIELDS, false);
        assert_eq!(ids(&predicate), [(0, 1), (0, 3), (0, 2)]);
    }

    #[test]
    fn a_compound_row_nests_one_rule_and_disappears_with_its_last_rule() {
        let mut predicate = Predicate {
            all: true,
            rules: vec![rule(1, 0, Operator::Is, "")],
        };
        predicate.insert_after(1, &FIELDS, true);
        assert_eq!(ids(&predicate), [(0, 1), (0, 2), (1, 3)]);
        predicate.insert_after(3, &FIELDS, false);
        assert_eq!(ids(&predicate), [(0, 1), (0, 2), (1, 3), (1, 4)]);
        predicate.remove(3);
        predicate.remove(4);
        assert_eq!(ids(&predicate), [(0, 1)]);
    }
}
