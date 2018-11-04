#![no_main]
#![no_std]

extern crate cortex_m_rt as rt;
use rt::entry;
use rt::exception;
use rt::ExceptionFrame;

extern crate cortex_m as cm;

#[macro_use(hprintln)]
extern crate cortex_m_semihosting;

extern crate panic_semihosting;

extern crate stm32f0;
use stm32f0::stm32f0x0;

#[entry]
fn main() -> ! {
    let peripherals = stm32f0x0::Peripherals::take().unwrap();
    let gpioa = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;

    // enable GPIOA peripheral clock
    rcc.ahbenr.write(|w| w.iopaen().set_bit());

    // configure PA5 as output pin
    gpioa.moder.write(|w| w.moder5().output());

    // configure PA5 pin as pull-down
    gpioa.pupdr.write(|w| w.pupdr5().pull_down());

    loop {
        hprintln!("Hello World!").unwrap();

        gpioa.odr.write(|w| w.odr5().set_bit());
        delay(3000);
        gpioa.odr.write(|w| w.odr5().clear_bit());
        delay(1000);
    }
}

fn delay(count: u32) {
    for _ in 0..count {
        cm::asm::nop();
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
