use gd32vf103xx_hal::delay::McycleDelay;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::led::{rgb, Led};
use longan_nano::sprintln;

#[inline(never)]
#[panic_handler]
fn on_panic(info: &core::panic::PanicInfo) -> ! {
    sprintln!("{}", info);

    let dp = pac::Peripherals::take().unwrap();
    let mut rcu = dp.RCU.configure().freeze();

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpioc = dp.GPIOC.split(&mut rcu);

    let (mut red, mut green, mut blue) = rgb(gpioc.pc13, gpioa.pa1, gpioa.pa2);
    let leds: [&mut dyn Led; 3] = [&mut red, &mut green, &mut blue];

    let mut delay = McycleDelay::new(&rcu.clocks);
    let mut i = 0;
    loop {
        let inext = (i + 1) % leds.len();
        leds[i].off();
        leds[inext].on();
        delay.delay_ms(500);

        i = inext;
    }
}
