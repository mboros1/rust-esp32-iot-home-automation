mod wifi;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{delay::FreeRtos, prelude::Peripherals},
    http::{server::EspHttpServer, Method},
    io::{EspIOError, Write},
    nvs::EspDefaultNvsPartition,
};
use rust_embed::Embed;
use wifi::wifi_connect;

#[derive(Embed)]
#[folder = "templates/"]
pub struct Assets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    for file in Assets::iter() {
        println!("Embedded file: {}", file.as_ref());
    }

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = wifi_connect("chaos central", "tinypotato392", peripherals.modem, sysloop)?;

    // Start Wifi
    wifi.start()?;

    // Connect Wifi
    wifi.connect()?;

    // Set the HTTP server
    let mut server = EspHttpServer::new(&esp_idf_svc::http::server::Configuration::default())?;
    // http://<sta ip>/ handler
    server.fn_handler(
        "/",
        Method::Get,
        |request| -> core::result::Result<(), EspIOError> {
            let html = index_html();
            let mut response = request.into_ok_response()?;
            response.write_all(html.as_bytes())?;
            Ok(())
        },
    )?;

    loop {
        FreeRtos::delay_ms(3000);
    }
}

fn index_html() -> String {
    if let Some(file) = Assets::get("index.html") {
        // The embedded file's data is stored as bytes. Convert it to a string.
        std::str::from_utf8(file.data.as_ref())
            .expect("Failed to convert asset data to UTF-8")
            .to_string()
    } else {
        "Missing HTML file".to_string()
    }
}
