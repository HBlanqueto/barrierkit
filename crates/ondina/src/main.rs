mod albums;
mod localize;
mod views;

use std::rc::Rc;
use std::time::Duration;

use barrier_kit::adaptive::{Breakpoint, use_breakpoint};
use barrier_kit::app::{Action, Instance, Launch, Shortcut, ShortcutKey, launch, use_menu_action};
use barrier_kit::app_menu::{AppMenu, Menu};
use barrier_kit::backdrop::{Accent, Backdrop, BackdropSource, Tint};
use barrier_kit::compact::IconButton;
use barrier_kit::controls::ScrollView;
use barrier_kit::platform::{FullscreenBar, Timers, WindowControls, WindowOptions, run_window};
use barrier_kit::sidebar::{Sidebar, SidebarBackdrop, SidebarItem};
use barrier_kit::symbol::Symbol;
use barrier_kit::titlebar::Titlebar;
use barrier_kit::views::split::{SidebarToggle, SplitView};
use barrier_kit::views::tabs::TabView;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Content, Gaps, Size};
use vow_design::{ACCENT_SWATCHES, Rgba, WINDOW_MIN_SIZE};

use albums::{Album, library};
use localize::tr;

const TICK: Duration = Duration::from_millis(500);

const STUDIO: [Rgba; 3] = [
    Rgba::rgb8(59, 47, 122),
    Rgba::rgb8(29, 111, 126),
    Rgba::rgb8(138, 63, 110),
];

type Title = fn() -> &'static str;

pub const PRESETS: [(Title, [Rgba; 3]); 3] = [
    (
        || tr!("preset-sunset"),
        [
            Rgba::rgb8(122, 59, 94),
            Rgba::rgb8(45, 110, 106),
            Rgba::rgb8(180, 112, 63),
        ],
    ),
    (
        || tr!("preset-forest"),
        [
            Rgba::rgb8(30, 77, 43),
            Rgba::rgb8(120, 160, 90),
            Rgba::rgb8(70, 60, 110),
        ],
    ),
    (
        || tr!("preset-midnight"),
        [
            Rgba::rgb8(20, 33, 61),
            Rgba::rgb8(58, 79, 122),
            Rgba::rgb8(150, 110, 40),
        ],
    ),
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tab {
    Now,
    Library,
    Mix,
    Look,
}

impl Tab {
    pub const ALL: [Self; 4] = [Self::Now, Self::Library, Self::Mix, Self::Look];

    pub fn title(self) -> &'static str {
        match self {
            Self::Now => tr!("tab-now"),
            Self::Library => tr!("tab-library"),
            Self::Mix => tr!("tab-mix"),
            Self::Look => tr!("tab-look"),
        }
    }

    pub fn symbol(self) -> Symbol {
        match self {
            Self::Now => Symbol::Play,
            Self::Library => Symbol::SquareGrid,
            Self::Mix => Symbol::SlidersVertical,
            Self::Look => Symbol::CircleHalf,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Background {
    Cover,
    Palette,
    Plain,
}

impl Background {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Self {
        [Self::Cover, Self::Palette, Self::Plain][index.min(2)]
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Player {
    pub album: State<usize>,
    pub track: State<usize>,
    pub playing: State<bool>,
    pub position: State<f32>,
    pub volume: State<f32>,
    albums: State<Rc<Vec<Album>>>,
}

impl Player {
    pub fn toggle(mut self) {
        let playing = *self.playing.peek();
        self.playing.set(!playing);
    }

    pub fn skip(mut self, step: isize) {
        let tracks = self.albums.peek()[*self.album.peek()].tracks.len() as isize;
        let next = (*self.track.peek() as isize + step).rem_euclid(tracks);
        self.track.set(next as usize);
        self.position.set(0.0);
    }

    fn advance(mut self) {
        if !*self.playing.peek() {
            return;
        }
        let albums = self.albums.peek().clone();
        let seconds = albums[*self.album.peek()].seconds(*self.track.peek()) as f32;
        let next = *self.position.peek() + TICK.as_secs_f32() / seconds;
        if next < 1.0 {
            self.position.set(next);
        } else {
            self.skip(1);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Mood {
    pub background: State<Background>,
    pub preset: State<usize>,
    pub accent: State<usize>,
    pub own_accent: State<usize>,
    pub controls: State<usize>,
    pub own_controls: State<usize>,
    pub eq: State<[f32; 5]>,
    pub switches: State<[bool; 3]>,
    pub checks: State<[bool; 2]>,
    pub quality: State<usize>,
    pub export: State<f32>,
    pub sidebar: State<SidebarBackdrop>,
}

impl Mood {
    pub fn tint(self) -> Tint {
        match *self.controls.read() {
            1 => Tint::Backdrop,
            2 => Tint::Color(ACCENT_SWATCHES[*self.own_controls.read()].1),
            _ => Tint::Accent,
        }
    }

    fn accent(self) -> Accent {
        match *self.accent.read() {
            1 => Accent::FromBackdrop,
            2 => Accent::Own(ACCENT_SWATCHES[*self.own_accent.read()].1),
            _ => Accent::System,
        }
    }
}

const PLAY_PAUSE: Action = Action::new("play-pause", || tr!("play-pause"))
    .shortcut(Shortcut::ctrl(ShortcutKey::Char('p')));
const NEXT: Action =
    Action::new("next", || tr!("next")).shortcut(Shortcut::ctrl(ShortcutKey::Right));
const PREVIOUS: Action =
    Action::new("previous", || tr!("previous")).shortcut(Shortcut::ctrl(ShortcutKey::Left));
const VIEWS: [Action; 4] = [
    Action::new("tab-now", || tr!("tab-now")).shortcut(Shortcut::ctrl(ShortcutKey::Char('1'))),
    Action::new("tab-library", || tr!("tab-library"))
        .shortcut(Shortcut::ctrl(ShortcutKey::Char('2'))),
    Action::new("tab-mix", || tr!("tab-mix")).shortcut(Shortcut::ctrl(ShortcutKey::Char('3'))),
    Action::new("tab-look", || tr!("tab-look")).shortcut(Shortcut::ctrl(ShortcutKey::Char('4'))),
];

fn menu() -> AppMenu {
    AppMenu::new(vec![
        Menu::new(
            tr!("menu-controls"),
            vec![PLAY_PAUSE.item(true), NEXT.item(true), PREVIOUS.item(true)],
        ),
        Menu::new(
            tr!("menu-view"),
            VIEWS.iter().map(|view| view.item(true)).collect(),
        ),
    ])
}

struct Ondina {
    instance: Option<Instance>,
}

impl App for Ondina {
    fn render(&self) -> impl IntoElement {
        let albums = use_state(|| Rc::new(library()));
        let player = Player {
            album: use_state(|| 1),
            track: use_state(|| 0),
            playing: use_state(|| true),
            position: use_state(|| 0.18),
            volume: use_state(|| 0.72),
            albums,
        };
        let mood = Mood {
            background: use_state(|| Background::Cover),
            preset: use_state(|| 0),
            accent: use_state(|| 1),
            own_accent: use_state(|| 5),
            controls: use_state(|| 0),
            own_controls: use_state(|| 4),
            eq: use_state(|| [0.55, 0.62, 0.5, 0.44, 0.6]),
            switches: use_state(|| [true, true, false]),
            checks: use_state(|| [true, false]),
            quality: use_state(|| 1),
            export: use_state(|| 0.375),
            sidebar: use_state(SidebarBackdrop::default),
        };
        let tab = use_state(|| Tab::Now);
        let instance = self.instance.clone();
        use_hook(move || {
            if let Some(instance) = &instance {
                instance.listen();
            }
            if let Some(timers) = try_consume_root_context::<Timers>() {
                schedule(timers, player);
            }
        });
        use_menu_action(move |id| match id {
            "play-pause" => player.toggle(),
            "next" => player.skip(1),
            "previous" => player.skip(-1),
            view => {
                if let Some(index) = VIEWS.iter().position(|v| v.id == view) {
                    let mut tab = tab;
                    tab.set(Tab::ALL[index]);
                }
            }
        });

        let current = *tab.read();
        let source = match (current, *mood.background.read()) {
            (Tab::Mix, _) => BackdropSource::Colors(STUDIO),
            (_, Background::Cover) => {
                BackdropSource::Image(albums.read()[*player.album.read()].cover.clone())
            }
            (_, Background::Palette) => BackdropSource::Colors(PRESETS[*mood.preset.read()].1),
            (_, Background::Plain) => BackdropSource::None,
        };
        Backdrop::new(source, Window { player, mood, tab }).accent(mood.accent())
    }
}

#[derive(PartialEq)]
struct Window {
    player: Player,
    mood: Mood,
    tab: State<Tab>,
}

impl Component for Window {
    fn render(&self) -> impl IntoElement {
        let (player, mood, tab) = (self.player, self.mood, self.tab);
        let mut queue_open = use_state(|| false);
        let size = use_breakpoint();
        let albums = player.albums.read().clone();
        let current = *tab.read();
        let album = &albums[*player.album.read()];
        let pane = match current {
            Tab::Now => views::now_playing(&albums, player, mood.tint()),
            Tab::Library => views::library(&albums, player),
            Tab::Mix => views::mixer(mood, album),
            Tab::Look => views::appearance(mood),
        };
        let pane = ScrollView::new()
            .width(Size::fill())
            .height(Size::flex(1.0))
            .child(pane);
        let breakpoint = size.current();
        let compact = breakpoint == Breakpoint::Compact;
        let queue_shown = breakpoint == Breakpoint::Wide || *queue_open.read();

        let mut titlebar = Titlebar::new()
            .title(current.title())
            .with_sidebar(!compact);
        if !compact {
            titlebar = titlebar.left(SidebarToggle);
        }
        if breakpoint != Breakpoint::Wide {
            titlebar = titlebar.right(
                IconButton::new(Symbol::ListBullet, tr!("queue"))
                    .on_press(move |_| queue_open.toggle()),
            );
        }
        let content = if compact {
            Tab::ALL
                .iter()
                .fold(
                    TabView::new(tr!("sections"))
                        .selected(Tab::ALL.iter().position(|t| *t == current).unwrap_or(0))
                        .on_change(move |index| {
                            let mut tab = tab;
                            tab.set(Tab::ALL[index]);
                        }),
                    |view, &item| {
                        let shown = if item != current {
                            rect().into_element()
                        } else if queue_shown {
                            views::queue(&albums, player, false)
                        } else {
                            pane.clone().into_element()
                        };
                        view.tab(item.symbol(), item.title(), shown)
                    },
                )
                .into_element()
        } else {
            pane.into_element()
        };
        let content_top = try_consume_root_context::<WindowControls>()
            .map_or(0.0, |controls| controls.fullscreen_content_top());
        let detail = rect()
            .width(Size::fill())
            .height(Size::fill())
            .content(Content::flex())
            .child(titlebar)
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .padding(Gaps::new(content_top, 0.0, 0.0, 0.0))
                    .child(content),
            );
        if compact {
            return size.attach(detail);
        }

        let sidebar = Sidebar::new()
            .traffic_lights(true)
            .backdrop(*mood.sidebar.read())
            .children(Tab::ALL.iter().enumerate().map(|(index, &item)| {
                SidebarItem::new(item.title())
                    .symbol(item.symbol(), ACCENT_SWATCHES[index].1)
                    .selected(item == current)
                    .on_select(move |_| {
                        let mut tab = tab;
                        tab.set(item);
                    })
            }));
        size.attach(
            rect().width(Size::fill()).height(Size::fill()).child(
                SplitView::new(sidebar, detail)
                    .inspector(views::queue(&albums, player, true), queue_shown),
            ),
        )
    }
}

fn schedule(timers: Timers, player: Player) {
    let again = timers.clone();
    timers.after(TICK, move || {
        player.advance();
        schedule(again.clone(), player);
    });
}

fn main() {
    barrier_kit::i18n::use_app_language("ondina");
    barrier_kit::init_tracing();
    let instance = match launch(&barrier_kit::titlebar::app_id("ondina")) {
        Ok(Launch::Forwarded) => return,
        Ok(Launch::Primary(instance)) => Some(instance),
        Err(error) => {
            tracing::warn!(%error, "ondina: sin instancia única");
            None
        }
    };
    run_window(
        AppComponent::new(Ondina { instance }),
        WindowOptions::new("Ondina", "ondina")
            .size((980, 640))
            .min_size(Some(WINDOW_MIN_SIZE))
            .menu(menu())
            .fullscreen_bar(FullscreenBar::ProgressiveGlass),
    )
    .expect("ondina");
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;

    fn labels(width: f32) -> Vec<String> {
        let app = AppComponent::new(Ondina { instance: None });
        let (mut test, _) = TestingRunner::new(app, Size2D::new(width, 640.0), |_| {}, 1.0);
        test.sync_and_update();
        test.poll(
            std::time::Duration::from_millis(16),
            std::time::Duration::from_millis(600),
        );
        if let Some(path) = std::env::var_os("ONDINA_SHOT") {
            test.render_to_file(std::path::Path::new(&path).join(format!("ondina-{width}.png")));
        }
        test.find_many(|_, el| el.accessibility().builder.label().map(str::to_owned))
    }

    #[test]
    fn the_layout_changes_shape_with_the_window_width() {
        let wide = labels(1000.0);
        let medium = labels(760.0);
        let compact = labels(400.0);
        assert!(
            wide.iter().any(|l| l == "Bajamar" || l == "Cable pelado"),
            "the wide window shows the queue"
        );
        assert!(
            !medium.iter().any(|l| l == "Cable pelado"),
            "the medium window hides the queue"
        );
        assert!(
            medium.iter().any(|l| l == tr!("queue")),
            "the medium window offers a queue button"
        );
        assert_eq!(
            compact
                .iter()
                .filter(|l| Tab::ALL.iter().any(|t| t.title() == l.as_str()))
                .count(),
            4
        );
    }
}
