use ribbit::BitTemplate;

mod ribbit;

pub fn run() {
    let mut app = bits_helpers::get_default_app::<BitTemplate>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    app.run();
}
