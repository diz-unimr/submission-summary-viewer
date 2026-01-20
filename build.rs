#[allow(clippy::expect_used)]
fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        winresource::WindowsResource::new()
            .set_icon("./resources/icon.ico")
            .compile()
            .expect("windows resources compiled");
    }
}
