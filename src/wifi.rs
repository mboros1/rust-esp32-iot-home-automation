use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    sys::{EspError, ESP_ERR_INVALID_ARG},
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, EspWifi},
};

pub fn wifi_connect(
    ssid: &str,
    pass: &str,
    modem: impl esp_idf_svc::hal::peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>, EspError> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        log::error!("Missing WiFi name");
        return Err(EspError::from(ESP_ERR_INVALID_ARG).unwrap());
    }
    if pass.is_empty() {
        auth_method = AuthMethod::None;
        log::info!("Wifi password is empty");
    }
    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take().ok();
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), nvs)?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::Client(
        ClientConfiguration::default(),
    ))?;

    log::info!("Starting wifi...");

    wifi.start()?;

    log::info!("Scanning...");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        log::info!(
            "Found configured access point {} on channel {}",
            ssid,
            ours.channel
        );
        Some(ours.channel)
    } else {
        log::info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::Client(
        ClientConfiguration {
            ssid: ssid
                .try_into()
                .expect("Could not parse the given SSID into WiFi config"),
            password: pass
                .try_into()
                .expect("Could not parse the given password into WiFi config"),
            channel,
            auth_method,
            ..Default::default()
        },
    ))?;

    log::info!("Connecting wifi...");

    wifi.connect()?;

    log::info!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    log::info!("Wifi DHCP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}
