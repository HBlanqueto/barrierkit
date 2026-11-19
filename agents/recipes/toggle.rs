use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("network")
        .window("Network", || {
            let mut wifi = use_state(|| true);
            VStack::new()
                .child(
                    Switch::new("Wi-Fi")
                        .on(*wifi.read())
                        .on_toggle(move |on| wifi.set(on)),
                )
                .child(Text::new(if *wifi.read() { "Connected" } else { "Off" }))
        })
        .run()
}
