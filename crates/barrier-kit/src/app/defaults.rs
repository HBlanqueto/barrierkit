use std::io;
use std::path::PathBuf;

use freya_core::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Clone, Copy)]
pub(crate) struct DefaultsRevision(pub(crate) State<u64>);

pub fn use_defaults<T>(app_id: &'static str) -> Memo<T>
where
    T: DeserializeOwned + Default + PartialEq + Clone + 'static,
{
    let revision = try_consume_root_context::<DefaultsRevision>();
    use_memo(move || {
        if let Some(DefaultsRevision(revision)) = revision {
            revision.read();
        }
        load_defaults(app_id)
    })
}

fn domain(app_id: &str) -> PathBuf {
    vow_appearance::config_dir().join(format!("{app_id}.domain"))
}

pub fn load_defaults<T: DeserializeOwned + Default>(app_id: &str) -> T {
    std::fs::read_to_string(domain(app_id))
        .ok()
        .and_then(|text| toml::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_defaults<T: Serialize>(app_id: &str, value: &T) -> io::Result<()> {
    let path = domain(app_id);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = toml::to_string(value).map_err(io::Error::other)?;
    let staged = path.with_extension("domain.tmp");
    std::fs::write(&staged, text)?;
    std::fs::rename(staged, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    static CONFIG_DIR: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    struct Prefs {
        step: i32,
    }

    #[test]
    fn use_defaults_reloads_the_domain_when_the_revision_moves() {
        use freya_core::integration::{App, AppComponent};
        use freya_testing::TestingRunner;
        use torin::prelude::Size2D;

        struct Reader {
            seen: State<i32>,
        }
        impl App for Reader {
            fn render(&self) -> impl IntoElement {
                let prefs = use_defaults::<Prefs>("demo.watched");
                let mut seen = self.seen;
                seen.set_if_modified(prefs.read().step);
                rect()
            }
        }

        let _guard = CONFIG_DIR.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("barrierkit-watched-{}", std::process::id()));
        // SAFETY: CONFIG_DIR serialises the tests that touch this variable.
        unsafe { std::env::set_var("MALAYSIA_CONFIG_DIR", &dir) };
        save_defaults("demo.watched", &Prefs { step: 1 }).unwrap();
        let seen = State::create_global(0);
        let revision = State::create_global(0u64);
        let app = AppComponent::new(Reader { seen });
        let (mut test, _) = TestingRunner::new(
            app,
            Size2D::new(10.0, 10.0),
            move |runner| {
                runner.provide_root_context(move || DefaultsRevision(revision));
            },
            1.0,
        );
        test.sync_and_update();
        assert_eq!(*seen.peek(), 1);

        save_defaults("demo.watched", &Prefs { step: 7 }).unwrap();
        let mut bump = revision;
        *bump.write() += 1;
        test.poll_n(std::time::Duration::from_millis(16), 3);
        assert_eq!(*seen.peek(), 7);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn saved_defaults_load_back_and_a_missing_domain_gives_the_default() {
        let _guard = CONFIG_DIR.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("barrierkit-defaults-{}", std::process::id()));
        // SAFETY: CONFIG_DIR serialises the tests that touch this variable.
        unsafe { std::env::set_var("MALAYSIA_CONFIG_DIR", &dir) };
        assert_eq!(load_defaults::<Prefs>("demo.missing"), Prefs::default());
        save_defaults("demo.saved", &Prefs { step: 5 }).unwrap();
        assert_eq!(load_defaults::<Prefs>("demo.saved"), Prefs { step: 5 });
        let _ = std::fs::remove_dir_all(dir);
    }
}
