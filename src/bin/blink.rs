#![no_main]
#![no_std]

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
extern crate cortex_m as cm;

extern crate cortex_m_semihosting as sh;
extern crate panic_semihosting;
extern crate stm32f0;

use core::fmt::Write;
use rt::ExceptionFrame;
use sh::hio;
use stm32f0::stm32f0x0;

entry!(main);

fn main() -> ! {
    let mut stdout = hio::hstdout().unwrap();

    let peripherals = stm32f0x0::Peripherals::take().unwrap();
    let gpioa = &peripherals.GPIOA;
    let rcc = &peripherals.RCC;

    // enable GPIOA peripheral clock
    rcc.ahbenr.modify(|_, w| w.iopaen().set_bit());

    // configure PA5 as output pin
    gpioa.moder.modify(|_, w| w.moder5().output());

    // configure PA5 pin as pull-down
    gpioa.pupdr.modify(|_, w| w.pupdr5().pull_down());

    loop {
        writeln!(stdout, "Hello World!").unwrap();

        gpioa.odr.modify(|_, w| w.odr5().set_bit());
        delay(3000);
        gpioa.odr.modify(|_, w| w.odr5().clear_bit());
        delay(1000);
    }
}

fn delay(count: u32) {
    for _ in 0..count {
        cm::asm::nop();
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
