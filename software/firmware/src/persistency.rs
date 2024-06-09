//use embassy_embedded_hal::flash;
// use embassy_rp::flash::Flash;
// use embassy_rp::peripherals::FLASH;

// fn dummy() {
//     let p = embassy_rp::init(Default::default());
//     let mut flash: Flash<'_, FLASH, embassy_rp::flash::Async, 42> = Flash::new(p.FLASH, p.DMA_CH0);
//     let mut data: [u8; 2] = [0; 2];
//     let result = flash.blocking_read(0, &mut data);
// }
