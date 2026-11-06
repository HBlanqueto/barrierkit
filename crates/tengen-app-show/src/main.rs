mod animation;
mod chrome;
mod events;
mod kit;
mod localize;
mod motion;
mod slides;

use std::sync::LazyLock;

use barrier_kit::adaptive::{Breakpoint, use_breakpoint};
use barrier_kit::adornments::{Link, avatar, badge, divider};
use barrier_kit::app::{Action, Instance, Launch, launch, load_defaults};
use barrier_kit::app::{REDO, UNDO, UndoManager, use_undo};
use barrier_kit::app::{Shortcut, ShortcutKey};
use barrier_kit::app_menu::{AppMenu, Menu};
use barrier_kit::backdrop::{Accent, Backdrop, BackdropSource, Tint};
use barrier_kit::choices::{ChoiceCards, ColorWell, RadioGroup};
use barrier_kit::combo_box::ComboBox;
use barrier_kit::compact::{Face, IconButton, Stepper, ToggleGroup};
use barrier_kit::context_menu::context_menu;
use barrier_kit::controls::{
    Button, ButtonStyle, Checkbox, ScrollView, SearchField, SecureField, SegmentedControl, Slider,
    Switch, TextField, ToolGroup,
};
use barrier_kit::level::{LevelSlider, LevelStyle};
use barrier_kit::menu::MenuItem;
use barrier_kit::password::PasswordField;
use barrier_kit::patterns::about::about_window;
use barrier_kit::patterns::language::app_language_row;
use barrier_kit::patterns::preferences::{Pane, PaneGroup, PreferencesWindow};
use barrier_kit::platform::{WindowControls, WindowStatus};
use barrier_kit::platform::{WindowOptions, run_window};
use barrier_kit::popover::show_popover;
use barrier_kit::popup_button::PopUpButton;
use barrier_kit::progress::{ProgressBar, ProgressRing, Spinner};
use barrier_kit::sidebar::{Sidebar, SidebarBackdrop, SidebarItem};
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::Symbol;
use barrier_kit::text_editor::TextEditor;
use barrier_kit::theme::use_appearance;
use barrier_kit::titlebar::Titlebar;
use barrier_kit::tooltip::use_tooltip;
use barrier_kit::traffic::TrafficLights;
use barrier_kit::views::collection::CollectionView;
use barrier_kit::views::feedback::{Alert, empty_state};
use barrier_kit::views::form::{Form, Row, Section};
use barrier_kit::views::table::{SortOrder, Table};
use barrier_kit::views::tabs::TabView;
use chrome::Prefs;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use localize::tr;
use torin::prelude::{Area, Direction, Size};
use vow_appearance::ColorScheme;
use vow_design::{
    ACCENT_SWATCHES, COLLECTION, Rgba, SPACE_2, SPLIT_VIEW, TABLE, TEXT, TEXT_FIELD,
    WINDOW_MIN_SIZE,
};

const APP: &str = "tengen-app-show";

fn tint(index: usize) -> Rgba {
    ACCENT_SWATCHES[index % ACCENT_SWATCHES.len()].1
}

fn row_of(children: impl IntoIterator<Item = Element>) -> Rect {
    rect()
        .direction(Direction::Horizontal)
        .spacing(SPACE_2)
        .children(children)
}

fn buttons() -> Form {
    Form::new().section(
        Section::titled(tr!("buttons"))
            .row(
                Row::new(tr!("variants")).control(row_of([
                    Button::new(tr!("cancel")).into_element(),
                    Button::new(tr!("save"))
                        .style(ButtonStyle::Primary)
                        .into_element(),
                    Button::new(tr!("inactive")).disabled(true).into_element(),
                ])),
            )
            .row(Row::new(tr!("icon-button")).control(row_of([
                IconButton::new(Symbol::Plus, tr!("add")).into_element(),
                IconButton::new(Symbol::Minus, tr!("remove")).into_element(),
            ])))
            .row(Row::new(tr!("link")).control(Link::new(tr!("learn-more"))))
            .row(Row::new(tr!("tooltip")).control(TooltipDemo))
            .row(Row::new(tr!("context-menu")).control(context_menu(
                rect().child(text(
                    tr!("secondary-click-here"),
                    TEXT,
                    Palette::current().secondary_label,
                )),
                vec![
                    Action::new("copy", || tr!("copy"))
                        .shortcut(Shortcut::ctrl(ShortcutKey::Char('c')))
                        .item(true),
                    Action::new("paste", || tr!("paste"))
                        .shortcut(Shortcut::ctrl(ShortcutKey::Char('v')))
                        .item(false),
                    MenuItem::Separator,
                    Action::new("info", || tr!("get-info")).item(true),
                ],
                |id: String| tracing::info!(%id, "menú contextual"),
            ))),
    )
}

#[derive(PartialEq)]
struct TooltipDemo;

impl Component for TooltipDemo {
    fn render(&self) -> impl IntoElement {
        use_tooltip(tr!("add-account")).attach(IconButton::new(Symbol::Plus, tr!("add")))
    }
}

fn toggles() -> Form {
    Form::new().section(
        Section::titled(tr!("switches-and-checkboxes"))
            .row(
                Row::new(tr!("switch")).control(row_of([
                    Switch::new(tr!("off")).into_element(),
                    Switch::new(tr!("on")).on(true).into_element(),
                    Switch::new(tr!("inactive"))
                        .on(true)
                        .disabled(true)
                        .into_element(),
                ])),
            )
            .row(Row::new(tr!("checkbox")).control(Checkbox::new(tr!("remember-me")).checked(true)))
            .row(
                Row::new(tr!("radio")).control(
                    RadioGroup::new(tr!("scope"))
                        .option(tr!("windows-only"))
                        .option(tr!("whole-desktop")),
                ),
            ),
    )
}

fn choices() -> Form {
    Form::new().section(
        Section::titled(tr!("choice"))
            .row(
                Row::new(tr!("segmented")).control(
                    SegmentedControl::new(tr!("period"))
                        .segment(tr!("day"))
                        .segment(tr!("week"))
                        .segment(tr!("month")),
                ),
            )
            .row(
                Row::new(tr!("popup")).control(
                    PopUpButton::new(tr!("traffic-lights"))
                        .item(tr!("automatic"))
                        .item(tr!("classic"))
                        .item(tr!("accent")),
                ),
            )
            .row(
                Row::new(tr!("accent-color")).control(
                    ColorWell::new(tr!("accent-color"), &ACCENT_SWATCHES).selected(Some(0)),
                ),
            )
            .row(
                Row::new(tr!("cards")).control(
                    [tr!("light"), tr!("dark"), tr!("accent")]
                        .into_iter()
                        .enumerate()
                        .fold(ChoiceCards::new(tr!("theme")), |cards, (index, caption)| {
                            cards.option(
                                caption,
                                rect()
                                    .width(Size::fill())
                                    .height(Size::fill())
                                    .background(barrier_kit::theme::color(tint(index))),
                            )
                        }),
                ),
            )
            .row(
                Row::new(tr!("toggle-group")).control(
                    ToggleGroup::new(tr!("format"))
                        .item("B", Face::Bold, tr!("bold"), true)
                        .item("I", Face::Italic, tr!("italic"), false)
                        .item("U", Face::Underline, tr!("underline"), false),
                ),
            )
            .row(Row::new(tr!("stepper")).control(Stepper::new(tr!("cores"), 1, 16).value(4))),
    )
}

static LEVELS: LazyLock<[&str; 4]> =
    LazyLock::new(|| [tr!("none"), tr!("little"), tr!("medium"), tr!("much")]);

fn sliders() -> Form {
    Form::new().section(
        Section::titled(tr!("sliders"))
            .row(Row::new(tr!("stepped")).control(Slider::stepped(4).value(1).labels(&*LEVELS)))
            .row(
                Row::new(tr!("continuous")).control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(LevelSlider::new(tr!("brightness")).value(0.4)),
                ),
            )
            .row(
                Row::new(tr!("control-center")).control(
                    rect().width(Size::px(SPLIT_VIEW.inspector)).child(
                        LevelSlider::new(tr!("volume"))
                            .value(0.6)
                            .style(LevelStyle::Fat(Symbol::SpeakerWave)),
                    ),
                ),
            ),
    )
}

fn fields(
    name: State<String>,
    query: State<String>,
    password: State<PasswordField>,
    shell: State<String>,
    notes: State<String>,
) -> Form {
    Form::new().section(
        Section::titled(tr!("fields"))
            .row(
                Row::new(tr!("text-field")).control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(TextField::new(name, tr!("user-name"))),
                ),
            )
            .row(
                Row::new(tr!("search-field")).control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(SearchField::new(query, tr!("search"))),
                ),
            )
            .row(
                Row::new(tr!("password"))
                    .control(SecureField::new(password).width(SPLIT_VIEW.inspector)),
            )
            .row(
                Row::new(tr!("combo-box")).control(
                    ComboBox::new(shell, tr!("shell"))
                        .item("zsh")
                        .item("fish")
                        .item("bash")
                        .width(Size::px(SPLIT_VIEW.inspector)),
                ),
            )
            .row(Row::new(tr!("text-editor")).control(
                rect().width(Size::px(SPLIT_VIEW.inspector)).child(
                    TextEditor::new(notes, tr!("notes")).height(Size::px(TEXT_FIELD.height * 3.0)),
                ),
            )),
    )
}

fn window_parts() -> Form {
    let status = WindowStatus::default();
    Form::new()
        .section(
            Section::titled(tr!("window"))
                .row(Row::new(tr!("traffic-lights")).control(TrafficLights::new().focused(true)))
                .row(
                    Row::new(tr!("title-bar")).control(
                        rect()
                            .width(Size::px(SPLIT_VIEW.inspector))
                            .child(Titlebar::preview(status, tr!("document"), false)),
                    ),
                )
                .row(
                    Row::new(tr!("close-only")).control(
                        rect()
                            .width(Size::px(SPLIT_VIEW.inspector))
                            .child(Titlebar::preview(status, tr!("about"), true).close_only(true)),
                    ),
                ),
        )
        .section(
            Section::titled(tr!("toolbar")).row(
                Row::new(tr!("group")).control(
                    ToolGroup::new()
                        .child(IconButton::new(Symbol::Plus, tr!("add")))
                        .child(IconButton::new(Symbol::Minus, tr!("remove")).disabled(true)),
                ),
            ),
        )
        .section(
            Section::titled(tr!("scrolling")).row(
                Row::new(tr!("list")).control(
                    ScrollView::new()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .height(Size::px(TABLE.row_height * 4.0))
                        .child(rect().width(Size::fill()).children((1..=20).map(|n| {
                            rect()
                                .height(Size::px(TABLE.row_height))
                                .child(text(
                                    tr!("item-n", number = n),
                                    TEXT,
                                    Palette::current().label,
                                ))
                                .into_element()
                        }))),
                ),
            ),
        )
}

#[derive(PartialEq)]
struct AdaptiveDemo;

impl Component for AdaptiveDemo {
    fn render(&self) -> impl IntoElement {
        let size = use_breakpoint();
        let name = match size.current() {
            Breakpoint::Compact => tr!("compact"),
            Breakpoint::Medium => tr!("medium"),
            Breakpoint::Wide => tr!("wide"),
        };
        size.attach(rect().width(Size::fill()))
            .child(text(name, TEXT, Palette::current().label))
    }
}

static SCHEMES: LazyLock<[(Option<ColorScheme>, &str); 4]> = LazyLock::new(|| {
    [
        (None, tr!("system")),
        (Some(ColorScheme::Light), tr!("light")),
        (Some(ColorScheme::Dark), tr!("dark")),
        (Some(ColorScheme::DarkTint), tr!("tinted")),
    ]
});

fn appearance(scheme: State<Option<ColorScheme>>) -> Form {
    let selected = SCHEMES
        .iter()
        .position(|(value, _)| *value == *scheme.read())
        .unwrap_or_default();
    Form::new().section(
        Section::titled(tr!("catalog-appearance"))
            .row(Row::new(tr!("theme")).control(chrome::segmented(
                tr!("theme"),
                &SCHEMES.map(|(_, name)| name),
                selected,
                move |index| {
                    let mut scheme = scheme;
                    scheme.set(SCHEMES[index].0);
                },
            )))
            .row(app_language_row(APP)),
    )
}

fn adaptive() -> Form {
    Form::new().section(
        Section::titled(tr!("adaptive-layout"))
            .row(Row::new(tr!("panel-breakpoint")).control(AdaptiveDemo)),
    )
}

fn progress() -> Form {
    Form::new().section(
        Section::titled(tr!("progress"))
            .row(
                Row::new(tr!("bar")).control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(ProgressBar::determinate(tr!("copying"), 0.62)),
                ),
            )
            .row(
                Row::new(tr!("indeterminate")).control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(ProgressBar::indeterminate(tr!("searching"))),
                ),
            )
            .row(Row::new(tr!("progress-ring")).control(ProgressRing::new(tr!("downloading"), 0.4)))
            .row(Row::new(tr!("activity-indicator")).control(Spinner::new(tr!("loading")))),
    )
}

fn adornments() -> Form {
    Form::new().section(
        Section::titled(tr!("badges-and-adornments"))
            .row(Row::new(tr!("badge")).control(badge("2")))
            .row(Row::new(tr!("avatar")).control(avatar("HB")))
            .row(
                Row::new(tr!("separator")).control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .child(divider()),
                ),
            ),
    )
}

fn forms() -> Form {
    Form::new()
        .section(
            Section::titled(tr!("rows"))
                .row(
                    Row::new(tr!("with-control"))
                        .detail(tr!("detail-below"))
                        .control(Switch::new(tr!("demo"))),
                )
                .row(Row::value(tr!("value-row"), tr!("value-sample")))
                .row(Row::navigation(tr!("navigation-row"), |_| {}))
                .row(Row::destructive(tr!("destructive-row"), |_| {})),
        )
        .section(
            Section::new()
                .row(Row::new(tr!("untitled-section")))
                .footer(tr!("section-footer")),
        )
}

fn data(sort: State<Option<SortOrder>>, selected: State<Vec<usize>>) -> Form {
    let mut sort_write = sort;
    let mut selected_write = selected;
    let mut files = [
        ("adr-0047.pdf", "248 KB", 248_000),
        ("notas.txt", "4 KB", 4_000),
        ("portada.png", "2,3 MB", 2_300_000),
        ("roadmap.pdf", "132 KB", 132_000),
    ];
    if let Some(order) = *sort.read() {
        files.sort_by(|a, b| {
            let ordering = if order.column == 0 {
                a.0.cmp(b.0)
            } else {
                a.2.cmp(&b.2)
            };
            if order.ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
    }
    let rows = files
        .iter()
        .map(|(name, size, _)| vec![name.to_string(), size.to_string()])
        .collect();
    let folder = |index: usize| {
        rect()
            .width(Size::px(COLLECTION.icon))
            .height(Size::px(COLLECTION.icon))
            .corner_radius(COLLECTION.radius)
            .background(barrier_kit::theme::color(tint(index)))
    };
    Form::new()
        .section(
            Section::titled(tr!("table")).row(
                Row::new(tr!("files")).control(
                    rect()
                        .width(Size::px(SPLIT_VIEW.sidebar_max))
                        .height(Size::px(TABLE.header_height + 4.0 * TABLE.row_height))
                        .child(
                            Table::new(tr!("files"), vec![tr!("name"), tr!("size")])
                                .rows(rows)
                                .sort(*sort.read())
                                .on_sort(move |order| sort_write.set(Some(order))),
                        ),
                ),
            ),
        )
        .section(
            Section::titled(tr!("grid")).row(
                Row::new(tr!("folders")).control(
                    CollectionView::new(tr!("folders"))
                        .item(folder(0), "Malaysia")
                        .item(folder(1), "Facturas")
                        .item(folder(6), "Fotos")
                        .selected(selected.read().clone())
                        .on_select(move |items| selected_write.set(items)),
                ),
            ),
        )
}

fn tabs() -> Form {
    Form::new().section(
        Section::titled(tr!("tabs")).row(
            Row::new(tr!("toolbar")).control(
                rect()
                    .width(Size::px(SPLIT_VIEW.sidebar_max))
                    .height(Size::px(SPLIT_VIEW.inspector / 2.0))
                    .child(
                        TabView::new(tr!("sections"))
                            .tab(
                                Symbol::MagnifyingGlass,
                                tr!("search"),
                                text(tr!("search-page"), TEXT, Palette::current().label),
                            )
                            .tab(
                                Symbol::SpeakerWave,
                                tr!("sound"),
                                text(tr!("sound-page"), TEXT, Palette::current().label),
                            ),
                    ),
            ),
        ),
    )
}

fn backdrop_card(colors: [Rgba; 3], accent: Accent, content: Rect) -> Rect {
    rect()
        .width(Size::px(320.0))
        .height(Size::px(150.0))
        .corner_radius(12.0)
        .overflow(Overflow::Clip)
        .child(Backdrop::new(BackdropSource::Colors(colors), content).accent(accent))
}

fn preview(colors: [Rgba; 3], accent: Accent, fill: Tint) -> Rect {
    backdrop_card(
        colors,
        accent,
        rect()
            .width(Size::fill())
            .padding(16.0)
            .spacing(SPACE_2)
            .child(row_of([
                Switch::new(tr!("on")).on(true).tint(fill).into_element(),
                Checkbox::new(tr!("checked"))
                    .checked(true)
                    .tint(fill)
                    .into_element(),
            ]))
            .child(ProgressBar::determinate(tr!("progress"), 0.6).tint(fill))
            .child(LevelSlider::new(tr!("level")).value(0.4).tint(fill)),
    )
}

fn sidebar_preview(colors: [Rgba; 3], backdrop: SidebarBackdrop) -> Rect {
    backdrop_card(
        colors,
        Accent::FromBackdrop,
        rect().width(Size::px(160.0)).height(Size::fill()).child(
            Sidebar::new()
                .backdrop(backdrop)
                .child(
                    SidebarItem::new(tr!("listening"))
                        .symbol(Symbol::Play, tint(0))
                        .selected(true),
                )
                .child(SidebarItem::new(tr!("library")).symbol(Symbol::SquareGrid, tint(1))),
        ),
    )
}

fn backdrops() -> Form {
    let [(_, dusk), (_, night), (_, pale)] = *kit::SCENES;
    Form::new().section(
        Section::titled(tr!("tinted-backdrop"))
            .row(
                Row::new(tr!("backdrop-accent"))
                    .detail(tr!("backdrop-accent-detail"))
                    .control(preview(dusk, Accent::FromBackdrop, Tint::Accent)),
            )
            .row(
                Row::new(tr!("backdrop-gradient"))
                    .detail(tr!("backdrop-gradient-detail"))
                    .control(preview(night, Accent::System, Tint::Backdrop)),
            )
            .row(
                Row::new(tr!("light-palette"))
                    .detail(tr!("light-palette-detail"))
                    .control(preview(pale, Accent::Own(tint(4)), Tint::Accent)),
            )
            .row(
                Row::new(tr!("sidebar"))
                    .detail(tr!("sidebar-backdrop-detail"))
                    .control(sidebar_preview(dusk, SidebarBackdrop::Clear)),
            )
            .row(
                Row::new(tr!("tinted-island"))
                    .detail(tr!("tinted-island-detail"))
                    .control(sidebar_preview(dusk, SidebarBackdrop::Tinted)),
            ),
    )
}

fn feedback(mut alert: State<bool>) -> Form {
    Form::new().section(
        Section::titled(tr!("states-and-alerts"))
            .row(
                Row::new(tr!("empty"))
                    .control(empty_state(Symbol::MagnifyingGlass, tr!("no-results"))),
            )
            .row(
                Row::new(tr!("alert"))
                    .control(Button::new(tr!("show-alert")).on_press(move |_| alert.set(true))),
            )
            .row(Row::new(tr!("popover")).control(PopoverButton)),
    )
}

#[derive(PartialEq)]
struct PopoverButton;

impl Component for PopoverButton {
    fn render(&self) -> impl IntoElement {
        let mut area = use_state(Area::default);
        rect()
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(Button::new(tr!("show-popover")).on_press(move |_| {
                show_popover(
                    *area.peek(),
                    (SPLIT_VIEW.inspector, TEXT.line_height as f32 * 2.0),
                    || text(tr!("popover-text"), TEXT, Palette::current().label).into_element(),
                )
            }))
    }
}

fn undo(accent: UndoManager<usize>, selected: usize) -> Form {
    Form::new().section(
        Section::titled(tr!("undo"))
            .row(
                Row::new(tr!("color")).control(
                    ColorWell::new(tr!("color"), &ACCENT_SWATCHES)
                        .selected(Some(selected))
                        .on_change(move |index| accent.set(index)),
                ),
            )
            .row(
                Row::new(tr!("history")).control(row_of([
                    Button::new(UNDO.title())
                        .disabled(!accent.can_undo())
                        .on_press(move |_| accent.undo())
                        .into_element(),
                    Button::new(REDO.title())
                        .disabled(!accent.can_redo())
                        .on_press(move |_| accent.redo())
                        .into_element(),
                ])),
            ),
    )
}

fn about() -> Form {
    Form::new().section(
        Section::titled(tr!("about")).row(
            Row::new(tr!("window")).control(
                rect()
                    .width(Size::px(SPLIT_VIEW.sidebar_max))
                    .height(Size::px(SPLIT_VIEW.inspector))
                    .child(about_window(
                        avatar("MT"),
                        "Master Tengen App Show",
                        Some(tr!("version", version = "0.1")),
                        tr!("copyright"),
                    )),
            ),
        ),
    )
}

struct Show {
    app_id: &'static str,
    instance: Option<Instance>,
    prefs: Prefs,
}

impl App for Show {
    fn render(&self) -> impl IntoElement {
        let instance = self.instance.clone();
        use_hook(move || {
            if let Some(instance) = &instance {
                instance.listen();
            }
        });
        let name = use_state(|| "hblanqueto".to_string());
        let query = use_state(String::new);
        let password = use_state(PasswordField::default);
        let shell = use_state(|| "fish".to_string());
        let notes = use_state(String::new);
        let sort = use_state(|| None::<SortOrder>);
        let selected = use_state(Vec::new);
        let mut alert = use_state(|| false);
        let scheme = use_state(|| None::<ColorScheme>);
        let accent = use_state(|| 0);
        let accent_history = use_undo(accent);
        let initial = self.prefs;
        let prefs = use_state(|| initial);
        let reduced = use_state(|| barrier_kit::theme::resolved().reduced_motion);
        use_appearance(move |config| {
            if let Some(forced) = *scheme.read() {
                config.color_scheme = forced;
            }
            config.sidebar_style = prefs.read().sidebar_style();
            config.reduced_motion = *reduced.read();
        });
        let current = *prefs.read();
        if let Some(controls) = try_consume_root_context::<WindowControls>() {
            let (mut bar, mut push) = (controls.fullscreen_bar, controls.fullscreen_push);
            bar.set_if_modified(current.fullscreen_bar);
            push.set_if_modified(current.fullscreen_push);
        }

        let window = PreferencesWindow::new()
            .restoration(self.app_id)
            .suggestions(&["formularios", "campos", "interruptores"])
            .sidebar_resizable(current.sidebar_resizable())
            .sidebar_range(current.sidebar_range())
            .narrow_sidebar(current.narrow_sidebar)
            .group(
                PaneGroup::new(Some(tr!("layer-primitives")))
                    .pane(Pane::new(
                        "botones",
                        tr!("buttons"),
                        Symbol::Plus,
                        tint(0),
                        buttons(),
                    ))
                    .pane(Pane::new(
                        "interruptores",
                        tr!("switches"),
                        Symbol::ChevronRight,
                        tint(6),
                        toggles(),
                    ))
                    .pane(Pane::new(
                        "eleccion",
                        tr!("choice"),
                        Symbol::ChevronUpDown,
                        tint(1),
                        choices(),
                    ))
                    .pane(Pane::new(
                        "deslizadores",
                        tr!("sliders"),
                        Symbol::SpeakerWave,
                        tint(4),
                        sliders(),
                    ))
                    .pane(Pane::new(
                        "campos",
                        tr!("fields"),
                        Symbol::MagnifyingGlass,
                        tint(7),
                        fields(name, query, password, shell, notes),
                    ))
                    .pane(Pane::new(
                        "progreso",
                        tr!("progress"),
                        Symbol::ChevronDown,
                        tint(5),
                        progress(),
                    ))
                    .pane(Pane::new(
                        "adornos",
                        tr!("adornments"),
                        Symbol::XMark,
                        tint(2),
                        adornments(),
                    )),
            )
            .group(
                PaneGroup::new(Some(tr!("layer-views")))
                    .pane(Pane::new(
                        "formularios",
                        tr!("forms"),
                        Symbol::ChevronRight,
                        tint(0),
                        forms(),
                    ))
                    .pane(Pane::new(
                        "datos",
                        tr!("table-and-grid"),
                        Symbol::ChevronDown,
                        tint(1),
                        data(sort, selected),
                    ))
                    .pane(Pane::new(
                        "pestanas",
                        tr!("tabs"),
                        Symbol::ChevronUp,
                        tint(3),
                        tabs(),
                    ))
                    .pane(Pane::new(
                        "fondos",
                        tr!("tinted-backdrop"),
                        Symbol::CircleHalf,
                        tint(5),
                        backdrops(),
                    ))
                    .pane(Pane::new(
                        "estados",
                        tr!("states-and-alerts"),
                        Symbol::XMark,
                        tint(3),
                        feedback(alert),
                    ))
                    .pane(Pane::new(
                        "ventana",
                        tr!("window-and-tools"),
                        Symbol::SidebarLeft,
                        tint(6),
                        window_parts(),
                    ))
                    .pane(Pane::new(
                        "adaptable",
                        tr!("adaptive-layout"),
                        Symbol::SquareGrid,
                        tint(4),
                        adaptive(),
                    )),
            )
            .group(
                PaneGroup::new(Some(tr!("layer-patterns")))
                    .pane(Pane::new(
                        "acerca",
                        tr!("about"),
                        Symbol::ChevronLeft,
                        tint(7),
                        about(),
                    ))
                    .pane(Pane::new(
                        "apariencia",
                        tr!("appearance"),
                        Symbol::CircleHalf,
                        tint(0),
                        appearance(scheme),
                    )),
            )
            .group(chrome::group(prefs))
            .group(animation::group(reduced))
            .group(PaneGroup::new(Some(tr!("app-group"))).pane(Pane::new(
                "deshacer",
                tr!("undo"),
                Symbol::Backward,
                tint(2),
                undo(accent_history, *accent.read()),
            )));

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(chrome::window(current, window))
            .maybe(*alert.read(), |el| {
                el.child(
                    Alert::new(tr!("forget-network"), tr!("forget-network-message"))
                        .confirm(tr!("forget"), move |_| alert.set(false))
                        .cancel(tr!("cancel"), move |_| alert.set(false)),
                )
            })
    }
}

fn main() {
    barrier_kit::i18n::use_app_language(APP);
    barrier_kit::init_tracing();
    let app_id = barrier_kit::titlebar::app_id(APP).leak();
    let instance = match launch(app_id) {
        Ok(Launch::Forwarded) => return,
        Ok(Launch::Primary(instance)) => Some(instance),
        Err(error) => {
            tracing::warn!(%error, "tengen-app-show: sin instancia única");
            None
        }
    };
    let prefs: Prefs = load_defaults(APP);
    run_window(
        AppComponent::new(Show {
            app_id,
            instance,
            prefs,
        }),
        WindowOptions::new("Master Tengen App Show", APP)
            .size((980, 640))
            .min_size(Some(WINDOW_MIN_SIZE))
            .fullscreen_bar(prefs.fullscreen_bar)
            .fullscreen_push(prefs.fullscreen_push)
            .menu(AppMenu::new(vec![Menu::new(
                tr!("edit"),
                vec![UNDO.item(true), REDO.item(true)],
            )])),
    )
    .expect(APP);
}

#[cfg(test)]
mod tests {
    use super::*;
    use barrier_kit::chrome::{Reveal, TitlebarLayout};
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;
    use vow_design::TITLEBAR;

    fn light_groups(test: &TestingRunner) -> usize {
        let diameter = TITLEBAR.buttons.diameter as f32;
        let length = 3.0 * diameter + 2.0 * TITLEBAR.buttons.spacing as f32;
        let mut groups = test.find_many(|node, _| {
            let area = node.layout().area;
            let size = (area.width(), area.height());
            (size == (length, diameter) || size == (diameter, length))
                .then_some((area.min_x() as i32, area.min_y() as i32))
        });
        groups.sort_unstable();
        groups.dedup();
        groups.len()
    }

    fn render(prefs: Prefs) -> TestingRunner {
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Show {
                app_id: APP,
                instance: None,
                prefs,
            }),
            Size2D::new(900.0, 560.0),
            |_| {},
            1.0,
        );
        test.sync_and_update();
        test
    }

    #[test]
    fn every_layout_shows_the_traffic_lights_once() {
        for layout in [TitlebarLayout::Top, TitlebarLayout::Rail] {
            for flat_sidebar in [false, true] {
                let prefs = Prefs {
                    layout,
                    flat_sidebar,
                    ..Default::default()
                };
                assert_eq!(light_groups(&render(prefs)), 1, "{prefs:?}");
            }
        }
    }

    #[test]
    fn the_popup_hides_the_lights_until_the_pointer_reaches_the_edge() {
        for (layout, edge) in [
            (TitlebarLayout::Top, (450.0, 2.0)),
            (TitlebarLayout::Rail, (2.0, 280.0)),
        ] {
            let mut test = render(Prefs {
                layout,
                reveal: Reveal::Popup,
                ..Default::default()
            });
            assert_eq!(light_groups(&test), 0, "{layout:?} at rest");
            test.move_cursor(edge);
            test.poll(
                std::time::Duration::from_millis(16),
                std::time::Duration::from_millis(400),
            );
            assert_eq!(light_groups(&test), 1, "{layout:?} revealed");
        }
    }
}
