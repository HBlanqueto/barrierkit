mod localize;
mod sampler;

use std::f64::consts::PI;
use std::time::Duration;

use barrier_kit::charts::{Chart, Chart3D, HeatMap, Interpolation, MarkKind, SectorChart, Series};
use barrier_kit::controls::ScrollView;
use barrier_kit::platform::{Timers, WindowOptions, run_window};
use barrier_kit::style::{Palette, text};
use barrier_kit::symbol::Symbol;
use barrier_kit::theme::color;
use barrier_kit::titlebar::Titlebar;
use barrier_kit::views::table::{SortOrder, Table};
use barrier_kit::views::tabs::TabView;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Direction, Gaps, Size};
use vow_design::{CALLOUT, HEADLINE, LIST_GROUP, SPACE_2, SPACE_3, SPACE_4, WINDOW_MIN_SIZE};

use localize::tr;
use sampler::{Monitor, Process};

const SAMPLE: Duration = Duration::from_secs(1);

fn card(title: &'static str, content: impl IntoElement) -> Element {
    let palette = Palette::current();
    rect()
        .width(Size::fill())
        .direction(Direction::Vertical)
        .spacing(SPACE_2)
        .padding(Gaps::new_all(SPACE_3))
        .corner_radius(LIST_GROUP.radius)
        .background(color(palette.content_background))
        .border(
            Border::new()
                .width(vow_design::HAIRLINE)
                .alignment(BorderAlignment::Inner)
                .fill(color(palette.card_border)),
        )
        .child(text(title, HEADLINE, palette.label))
        .child(content)
        .into_element()
}

fn page(cards: impl IntoIterator<Item = Element>) -> Element {
    ScrollView::new()
        .child(
            rect()
                .width(Size::fill())
                .direction(Direction::Vertical)
                .spacing(SPACE_4)
                .padding(Gaps::new_all(SPACE_4))
                .children(cards),
        )
        .into_element()
}

fn wave(len: usize, phase: f64, amplitude: f64, base: f64) -> Vec<f64> {
    (0..len)
        .map(|i| {
            base + amplitude * (i as f64 * 0.55 + phase).sin()
                + (i as f64 * 1.7 + phase).cos() * amplitude / 3.0
        })
        .collect()
}

fn months() -> Vec<String> {
    ["E", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"]
        .map(String::from)
        .to_vec()
}

fn gallery() -> Element {
    let quarters: Vec<String> = ["T1", "T2", "T3", "T4"].map(String::from).to_vec();
    let heat: Vec<Vec<f64>> = (0..7)
        .map(|day| {
            (0..24)
                .map(|hour| ((hour as f64 - 13.0).powi(2) / -40.0).exp() * (1.0 + day as f64 % 3.0))
                .collect()
        })
        .collect();
    page([
        card(
            tr!("line-chart"),
            Chart::new(tr!("line-chart"), MarkKind::Line)
                .interpolation(Interpolation::Monotone)
                .series(Series::new(tr!("visits"), wave(12, 0.0, 30.0, 80.0)))
                .series(Series::new(tr!("sales"), wave(12, 1.3, 20.0, 50.0)))
                .x_labels(months()),
        ),
        card(
            tr!("area-chart"),
            Chart::new(tr!("area-chart"), MarkKind::Area)
                .stacked(true)
                .interpolation(Interpolation::Monotone)
                .series(Series::new(tr!("desktop"), wave(12, 0.4, 10.0, 40.0)))
                .series(Series::new(tr!("mobile"), wave(12, 2.0, 12.0, 30.0)))
                .x_labels(months()),
        ),
        card(
            tr!("bar-chart"),
            Chart::new(tr!("bar-chart"), MarkKind::Bar)
                .series(Series::new(tr!("north"), vec![42.0, 55.0, 61.0, 48.0]))
                .series(Series::new(tr!("south"), vec![30.0, 38.0, 52.0, 66.0]))
                .x_labels(quarters.clone())
                .rule(50.0),
        ),
        card(
            tr!("stacked-bars"),
            Chart::new(tr!("stacked-bars"), MarkKind::Bar)
                .stacked(true)
                .series(Series::new(tr!("sales"), vec![42.0, 55.0, 61.0, 48.0]))
                .series(Series::new(tr!("returns"), vec![8.0, 6.0, 11.0, 7.0]))
                .x_labels(quarters),
        ),
        card(
            tr!("point-chart"),
            Chart::new(tr!("point-chart"), MarkKind::Point)
                .series(Series::new(tr!("visits"), wave(24, 0.7, 25.0, 50.0)))
                .series(Series::new(tr!("sales"), wave(24, 2.9, 15.0, 30.0))),
        ),
        card(
            tr!("step-chart"),
            Chart::new(tr!("step-chart"), MarkKind::Line)
                .interpolation(Interpolation::Step)
                .series(Series::new(
                    tr!("visits"),
                    vec![2.0, 2.0, 5.0, 3.0, 8.0, 8.0, 6.0, 9.0, 4.0],
                )),
        ),
        card(
            tr!("pie-chart"),
            SectorChart::new(tr!("pie-chart"))
                .slice(tr!("photos"), 182.0)
                .slice(tr!("apps"), 96.0)
                .slice(tr!("system"), 64.0)
                .slice(tr!("other"), 40.0)
                .unit(" GB"),
        ),
        card(
            tr!("donut-chart"),
            SectorChart::new(tr!("donut-chart"))
                .donut(true)
                .slice(tr!("photos"), 182.0)
                .slice(tr!("apps"), 96.0)
                .slice(tr!("system"), 64.0)
                .slice(tr!("other"), 40.0)
                .unit(" GB"),
        ),
        card(
            tr!("heat-map"),
            HeatMap::new(tr!("heat-map"), heat).height(120.0),
        ),
    ])
}

fn sorted(processes: &[Process], sort: Option<SortOrder>) -> Vec<Vec<String>> {
    let mut list = processes.to_vec();
    if let Some(order) = sort {
        list.sort_by(|a, b| {
            let by = match order.column {
                0 => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                1 => a.pid.cmp(&b.pid),
                2 => a.memory.total_cmp(&b.memory),
                _ => a.threads.cmp(&b.threads),
            };
            if order.ascending { by } else { by.reverse() }
        });
    }
    list.iter()
        .map(|p| {
            vec![
                p.name.clone(),
                p.pid.to_string(),
                barrier_kit::charts::format_value(p.memory, " MB"),
                p.threads.to_string(),
            ]
        })
        .collect()
}

fn monitor_page(state: &Monitor, mut sort: State<Option<SortOrder>>) -> Element {
    let history =
        |values: &std::collections::VecDeque<f64>| values.iter().copied().collect::<Vec<_>>();
    let memory = state.memory;
    page([
        card(
            tr!("cpu-history"),
            Chart::new(tr!("cpu-history"), MarkKind::Area)
                .stacked(true)
                .y_domain(0.0, 100.0)
                .unit("%")
                .series(Series::new(tr!("cpu-user"), history(&state.user)))
                .series(Series::new(tr!("cpu-system"), history(&state.system))),
        ),
        card(
            tr!("cores"),
            HeatMap::new(tr!("cores"), state.cores.iter().map(history).collect())
                .height((state.cores.len() as f32 * 14.0).max(14.0)),
        ),
        card(
            tr!("memory"),
            SectorChart::new(tr!("memory"))
                .donut(true)
                .unit(" MB")
                .slice(tr!("memory-used"), memory.used)
                .slice(tr!("memory-cached"), memory.cached)
                .slice(tr!("memory-free"), memory.free),
        ),
        card(
            tr!("network"),
            Chart::new(tr!("network"), MarkKind::Line)
                .interpolation(Interpolation::Monotone)
                .unit(" KB/s")
                .series(Series::new(tr!("received"), history(&state.received)))
                .series(Series::new(tr!("sent"), history(&state.sent))),
        ),
        card(
            tr!("load"),
            Chart::new(tr!("load"), MarkKind::Bar)
                .series(Series::new(tr!("load"), state.load.to_vec()))
                .x_labels(vec![
                    tr!("load-1").into(),
                    tr!("load-5").into(),
                    tr!("load-15").into(),
                ])
                .height(120.0),
        ),
        card(
            tr!("processes"),
            Table::new(
                tr!("processes"),
                vec![
                    tr!("column-name"),
                    tr!("column-pid"),
                    tr!("column-memory"),
                    tr!("column-threads"),
                ],
            )
            .rows(sorted(&state.processes, *sort.read()))
            .sort(*sort.read())
            .on_sort(move |order: SortOrder| sort.set(Some(order))),
        ),
    ])
}

fn spaces() -> Element {
    let n = 28;
    let surface: Vec<Vec<f64>> = (0..n)
        .map(|r| {
            (0..n)
                .map(|c| {
                    let (x, z) = (
                        (c as f64 / (n - 1) as f64 - 0.5) * 4.0 * PI,
                        (r as f64 / (n - 1) as f64 - 0.5) * 4.0 * PI,
                    );
                    let d = x.hypot(z).max(1e-6);
                    d.sin() / d
                })
                .collect()
        })
        .collect();
    let mut seed = 7u64;
    let mut random = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as f64 / (1u64 << 31) as f64 * 2.0 - 1.0
    };
    let points: Vec<[f64; 3]> = (0..240)
        .map(|i| {
            let center = [[-0.5, -0.4, 0.3], [0.5, 0.5, -0.2], [0.1, -0.1, -0.6]][i % 3];
            [
                center[0] + random() * 0.35,
                center[1] + random() * 0.35,
                center[2] + random() * 0.35,
            ]
        })
        .collect();
    let palette = Palette::current();
    page([
        card(tr!("surface"), Chart3D::surface(tr!("surface"), surface)),
        card(tr!("scatter"), Chart3D::points(tr!("scatter"), points)),
        text(tr!("drag-hint"), CALLOUT, palette.secondary_label).into_element(),
    ])
}

struct Trazo {
    tab: usize,
}

impl App for Trazo {
    fn render(&self) -> impl IntoElement {
        let mut tab = use_state(|| self.tab);
        let mut monitor = use_state(Monitor::default);
        let sort = use_state(|| None::<SortOrder>);
        use_hook(move || {
            monitor.write().sample(SAMPLE.as_secs_f64());
            if let Some(timers) = try_consume_root_context::<Timers>() {
                schedule(timers, monitor);
            }
        });
        let current = *tab.read();
        let shown = |index: usize, content: fn() -> Element| {
            if index == current {
                content()
            } else {
                rect().into_element()
            }
        };
        let monitor_view = if current == 1 {
            monitor_page(&monitor.read(), sort)
        } else {
            rect().into_element()
        };
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Vertical)
            .child(Titlebar::new().title(tr!("app-name")))
            .child(
                TabView::new(tr!("sections"))
                    .selected(current)
                    .on_change(move |index| tab.set(index))
                    .tab(Symbol::SquareGrid, tr!("tab-2d"), shown(0, gallery))
                    .tab(Symbol::SlidersVertical, tr!("tab-monitor"), monitor_view)
                    .tab(Symbol::CircleHalf, tr!("tab-3d"), shown(2, spaces)),
            )
    }
}

fn schedule(timers: Timers, monitor: State<Monitor>) {
    let again = timers.clone();
    timers.after(SAMPLE, move || {
        let mut monitor = monitor;
        monitor.write().sample(SAMPLE.as_secs_f64());
        schedule(again.clone(), monitor);
    });
}

fn main() {
    barrier_kit::i18n::use_app_language("trazo");
    barrier_kit::init_tracing();
    run_window(
        AppComponent::new(Trazo { tab: 0 }),
        WindowOptions::new("Trazo", "trazo")
            .size((900, 720))
            .min_size(Some(WINDOW_MIN_SIZE)),
    )
    .expect("trazo");
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;

    #[test]
    fn every_tab_renders_its_charts() {
        for tab in 0..3 {
            let app = AppComponent::new(Trazo { tab });
            let (mut test, _) = TestingRunner::new(app, Size2D::new(900.0, 1400.0), |_| {}, 1.0);
            test.sync_and_update();
            if let Some(path) = std::env::var_os("TRAZO_SHOT") {
                test.render_to_file(std::path::Path::new(&path).join(format!("trazo-{tab}.png")));
            }
            let figures = test.find_many(|_, el| {
                (el.accessibility().builder.role() == AccessibilityRole::Figure).then_some(())
            });
            assert!(!figures.is_empty(), "tab {tab} has no chart");
        }
    }
}
