use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.network")
        .window("Network", || {
            let wifi = use_state(|| true);
            VStack::new()
                .child(Switch::new("Wi-Fi", wifi))
                .child(Text::new(if *wifi.read() { "Connected" } else { "Off" }))
        })
        .run()
}
