#![no_main]
#![no_std]

extern crate cortex_m_rt as rt;
use rt::entry;
use rt::exception;

extern crate cortex_m as cm;

extern crate stm32f0;
use stm32f0::stm32f0x0;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {
        cm::asm::bkpt();
    }
}

#[entry]
fn main() -> ! {
    let peripherals = stm32f0x0::Peripherals::take().unwrap();

    led2_init(&peripherals);

    loop {
        led2_blink(&peripherals, 3000, 1000);
    }
}

fn led2_init(p: &stm32f0x0::Peripherals) {
    let gpioa = &p.GPIOA;
    let rcc = &p.RCC;

    // enable GPIOA peripheral clock
    rcc.ahbenr.write(|w| w.iopaen().set_bit());

    // configure PA5 as output pin
    gpioa.moder.write(|w| w.moder5().output());

    // configure PA5 pin as pull-down
    gpioa.pupdr.write(|w| w.pupdr5().pull_down());
}

fn led2_blink(p: &stm32f0x0::Peripherals, t1: u32, t2: u32) {
    let gpioa = &p.GPIOA;

    loop {
        gpioa.odr.write(|w| w.odr5().set_bit());
        delay(t1);
        gpioa.odr.write(|w| w.odr5().clear_bit());
        delay(t2);
    }
}

fn delay(count: u32) {
    for _ in 0..count {
        cm::asm::nop();
    }
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
