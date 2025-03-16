use defmt::{unwrap, info};
use embassy_executor::{task, Spawner};
use embassy_rp::pio::Pio;
use embassy_rp::peripherals::{DMA_CH1, PIO1, PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_rp::gpio;
use static_cell::StaticCell;
use cyw43_pio::DEFAULT_CLOCK_DIVIDER;
use cyw43::JoinOptions;
use core::str;

use crate::drivers::persistency;
use crate::PersistencyMutexed;

pub struct WifiHw {
    pub pin_23: PIN_23,
    pub pin_24: PIN_24,
    pub pin_25: PIN_25,
    pub pin_29: PIN_29,
    pub pio_1: Pio<'static, PIO1>,
    pub dma_ch1: DMA_CH1,
}

#[task]
pub async fn run(persistency: &'static PersistencyMutexed, mut hw: WifiHw, spawner: Spawner) {
    let fw = include_bytes!("../../../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../../../cyw43-firmware/43439A0_clm.bin");

    let pwr = gpio::Output::new(hw.pin_23, gpio::Level::Low);
    let cs = gpio::Output::new(hw.pin_25, gpio::Level::High);
    let spi = cyw43_pio::PioSpi::new(&mut hw.pio_1.common, hw.pio_1.sm0, DEFAULT_CLOCK_DIVIDER, hw.pio_1.irq0, cs, hw.pin_24, hw.pin_29, hw.dma_ch1);

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner))); //TODO: irgenwie wird hier eine andere unwrap funktion verwendet. warum?

    control.init(clm).await;
    control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

    let mut ssid: [u8; 32] = ['\0' as u8; 32];
    let length = persistency.lock().await.read(persistency::ValueId::WifiSsid, &mut ssid).unwrap();
    let ssid = str::from_utf8(&ssid[..length]).unwrap();

    let mut password: [u8; 32] = ['\0' as u8; 32];
    let length = persistency.lock().await.read(persistency::ValueId::WifiPassword, &mut password).unwrap();
    let password = &password[..length];

    info!("ssid: {:?}", ssid);
    info!("password: {:?}", str::from_utf8(password).unwrap());

    loop {
        match control.join(ssid, JoinOptions::new(password)).await {
            Ok(_) => {
                info!("join successful");
                break
            },
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }
}

#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, gpio::Output<'static>, cyw43_pio::PioSpi<'static, PIO1, 0, DMA_CH1>>) -> ! {
    runner.run().await
}
