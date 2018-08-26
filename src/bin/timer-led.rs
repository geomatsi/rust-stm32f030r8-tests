#![no_main]
#![no_std]

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
extern crate cortex_m as cm;

extern crate cortex_m_semihosting as sh;
extern crate panic_semihosting;

#[macro_use(interrupt)]
extern crate stm32f0;

use core::fmt::Write;
use rt::ExceptionFrame;
use sh::hio;
use sh::hio::HStdout;
use stm32f0::stm32f0x0;

entry!(main);

fn main() -> ! {
    let mut core_periph = cm::peripheral::Peripherals::take().unwrap();
    let soc_periph = stm32f0x0::Peripherals::take().unwrap();

    setup_rcc(&soc_periph);
    setup_gpio(&soc_periph);
    setup_interrupts(&mut core_periph);
    setup_tim3(&soc_periph);

    start_tim3(&soc_periph);

    loop {
        delay(30000);
    }
}

fn setup_rcc(p: &stm32f0x0::Peripherals) {
    // enable GPIOA peripheral clock
    p.RCC.ahbenr.modify(|_, w| w.iopaen().set_bit());

    // enable TIM3 peripheral clock
    p.RCC.apb1enr.modify(|_, w| w.tim3en().set_bit());
}

fn setup_gpio(p: &stm32f0x0::Peripherals) {
    // configure PA5 as output pin
    p.GPIOA.moder.modify(|_, w| w.moder5().output());

    // configure PA5 pin as pull-down
    p.GPIOA.pupdr.modify(|_, w| w.pupdr5().pull_down());
}

fn setup_tim3(p: &stm32f0x0::Peripherals) {
    // TIM3: simple upcounting mode

    unsafe {
        // set timer start value
        p.TIM3.cnt.modify(|_, w| w.bits(1));

        // set timer prescaler: 8MHz/800 => 10000 ticks per second
        p.TIM3.psc.modify(|_, w| w.bits(800));

        // set timer value when interrupt is generated: once per second
        p.TIM3.arr.modify(|_, w| w.bits(10000));
    }

    // set timer value when interrupt is generated: once per second
    p.TIM3.dier.modify(|_, w| w.uie().set_bit());
}

fn start_tim3(p: &stm32f0x0::Peripherals) {
    p.TIM3.cr1.modify(|_, w| w.cen().set_bit());
}

fn setup_interrupts(p_core: &mut cm::peripheral::Peripherals) {
    let nvic = &mut p_core.NVIC;

    // Enable TIM3 IRQ, set prio 1 and clear any pending IRQs
    nvic.enable(stm32f0x0::Interrupt::TIM3);
    unsafe {
        nvic.set_priority(stm32f0x0::Interrupt::TIM3, 1);
    }
    nvic.clear_pending(stm32f0x0::Interrupt::TIM3);
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

interrupt!(TIM3, timer_tim3, state: Option<HStdout> = None);

fn timer_tim3(state: &mut Option<HStdout>) {
    if state.is_none() {
        *state = Some(hio::hstdout().unwrap());
    }

    unsafe {
        (*stm32f0::stm32f0x0::GPIOA::ptr())
            .odr
            .modify(|r, w| w.odr5().bit(!r.odr5().bit()));

        (*stm32f0::stm32f0x0::TIM3::ptr())
            .sr
            .modify(|_, w| w.uif().clear_bit());
    }

    if let Some(hstdout) = state.as_mut() {
        writeln!(hstdout, "TOGGLE").unwrap();
    }
}
