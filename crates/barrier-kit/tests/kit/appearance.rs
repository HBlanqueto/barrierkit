use barrier_kit::theme::{resolved, use_appearance};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::Size2D;
use vow_appearance::ColorScheme;

#[derive(PartialEq)]
struct Probe {
    seen: State<Vec<ColorScheme>>,
}

impl Component for Probe {
    fn render(&self) -> impl IntoElement {
        let mut seen = self.seen;
        seen.write().push(resolved().scheme);
        rect()
    }
}

#[derive(PartialEq)]
struct Forced {
    scheme: ColorScheme,
    seen: State<Vec<ColorScheme>>,
}

impl Component for Forced {
    fn render(&self) -> impl IntoElement {
        let scheme = self.scheme;
        use_appearance(move |config| config.color_scheme = scheme);
        rect().child(Probe { seen: self.seen })
    }
}

struct Host {
    scheme: ColorScheme,
    forced: State<Vec<ColorScheme>>,
    sibling: State<Vec<ColorScheme>>,
}

impl App for Host {
    fn render(&self) -> impl IntoElement {
        rect()
            .child(Forced {
                scheme: self.scheme,
                seen: self.forced,
            })
            .child(Probe { seen: self.sibling })
    }
}

#[test]
fn a_forced_appearance_reaches_the_subtree_and_not_its_siblings() {
    let system = vow_appearance::resolve(
        &vow_appearance::load_appearance(),
        vow_appearance::load_dominant(),
    );
    let scheme = if system.scheme == ColorScheme::DarkTint {
        ColorScheme::Light
    } else {
        ColorScheme::DarkTint
    };
    let forced = State::create_global(Vec::new());
    let sibling = State::create_global(Vec::new());
    let app = AppComponent::new(Host {
        scheme,
        forced,
        sibling,
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(10.0, 10.0), |_| {}, 1.0);
    test.sync_and_update();
    assert_eq!(forced.peek().last(), Some(&scheme));
    assert_ne!(sibling.peek().last(), Some(&scheme));
}
