#![no_std]
#![no_main]

use embedded_hal::spi::MODE_0;
use gd32vf103xx_hal::gpio::State;
use gd32vf103xx_hal::spi::Spi;
use riscv_rt::entry;

use longan_nano::hal::delay::McycleDelay;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::sprintln;

extern crate alloc;
use alloc::boxed::Box;

mod epd2in66b;
mod panic_blinky;

use core::ptr::NonNull;
use palloc::{GlobalPalloc, SpinPalloc};
#[global_allocator]
static mut ALLOCATOR: SpinPalloc = SpinPalloc::empty();

#[entry]
fn main() -> ! {
    unsafe { ALLOCATOR.init(NonNull::new(0x2000_0000 as *mut u8).unwrap(), 0x1FFF_FFFF) };

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
        // 921_600.bps(),
        &mut afio,
        &mut rcu,
    );

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

    let a = 0;
    sprintln!("pointer: {:p}", &a);
    // let a = Box::new(a);
    // sprintln!("pointer: {:p}", a.as_mut());
    loop {}

    delay.delay_ms(10);
    let mut epd = epd2in66b::Display::new(
        &mut spi1,
        gpioa.pa4.into_push_pull_output_with_state(State::High),
        gpioa.pa1.into_floating_input(),
        gpioc.pc13.into_push_pull_output(),
        gpioa.pa2.into_push_pull_output(),
        &mut delay,
    )
    .unwrap();
    epd.clear_frame(&mut spi1).unwrap();
    // Red - DC
    // Green - BUSY
    // Blue - RST
    loop {}
}
