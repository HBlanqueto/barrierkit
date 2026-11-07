use std::path::PathBuf;

#[test]
fn malaysia_dirs_follow_xdg_and_ignore_relative_values() {
    // SAFETY: this test binary has a single test, so no other thread reads the environment.
    unsafe {
        std::env::remove_var("MALAYSIA_CONFIG_DIR");
        std::env::remove_var("MALAYSIA_DATA_DIR");
        std::env::set_var("HOME", "/home/ana");
        std::env::set_var("XDG_CONFIG_HOME", "/xdg/config");
        std::env::set_var("XDG_DATA_HOME", "relative/data");
    }
    assert_eq!(
        ct_appearance::config_dir(),
        PathBuf::from("/xdg/config/malaysia")
    );
    assert_eq!(
        ct_appearance::data_dir(),
        PathBuf::from("/home/ana/.local/share/malaysia")
    );

    // SAFETY: as above.
    unsafe { std::env::set_var("XDG_DATA_DIRS", "relative:/opt/share") };
    assert_eq!(
        ct_appearance::xdg::data_dirs(),
        [PathBuf::from("/opt/share")]
    );
}
