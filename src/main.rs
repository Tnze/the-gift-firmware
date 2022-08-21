#![no_std]
#![no_main]
#![feature(associated_type_bounds)]

use core::panic::PanicInfo;

use ec01f::EC01F;
use embedded_hal::spi::MODE_0;
use gd32vf103xx_hal::serial::{Serial, Config, Parity, StopBits};
use gd32vf103xx_hal::spi::Spi;
use riscv_rt::entry;

use longan_nano::hal::delay::McycleDelay;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::sprintln;

mod ec01f;
mod epd2in66b;

use epd2in66b::{DeepSleepMode, Epd2in66bDisplay};

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    sprintln!("Panic: {}", info);
    loop {}
}

#[entry]
fn main() -> ! {
    // 国家授时中心 ntp.ntsc.ac.cn
    let dp = pac::Peripherals::take().unwrap();
    let mut rcu = dp.RCU.configure().freeze();

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpioc = dp.GPIOC.split(&mut rcu);
    let mut afio = dp.AFIO.constrain(&mut rcu);

    longan_nano::stdout::configure(
        dp.USART0,
        gpioa.pa9,
        gpioa.pa10,
        9600.bps(),
        &mut afio,
        &mut rcu,
    );
    sprintln!("program started");

    let mut serial1 = Serial::new(
        dp.USART1,
        (
            gpioa.pa2.into_push_pull_output(),
            gpioa.pa3.into_floating_input(),
        ),
        Config{
            baudrate: 9600.bps(),
            parity: Parity::ParityNone,
            stopbits: StopBits::STOP1
        },
        &mut afio,
        &mut rcu,
    );
    let (tx, rx) = serial1.split();
    let mut ec01f = EC01F::new(tx, rx);

    let mut delay = McycleDelay::new(&rcu.clocks);

    // 初始化SPI
    let mut spi1 = Spi::spi0(
        dp.SPI0,
        (
            gpioa.pa5.into_alternate_push_pull(),
            gpioa.pa6.into_floating_input(),
            gpioa.pa7.into_alternate_push_pull(),
        ),
        &mut afio,
        MODE_0,
        400_0000.hz(),
        &mut rcu,
    );

    delay.delay_ms(10);
    let mut epd = Epd2in66bDisplay::new(
        &mut spi1,
        gpioa.pa4.into_push_pull_output(),
        gpioa.pa0.into_floating_input(),
        gpioc.pc13.into_push_pull_output(),
        gpioa.pa1.into_push_pull_output(),
        &mut delay,
    )
    .unwrap();
    epd.clear_frame(&mut spi1).unwrap();
    // epd.test(&mut spi1).unwrap();
    epd.activate(&mut spi1).unwrap();
    epd.deep_sleep(&mut spi1, DeepSleepMode::Normal).unwrap();
    // Red - DC
    // Green - BUSY
    // Blue - RST
    loop {}
}
