#![no_main]
#![no_std]

extern crate cortex_m_rt as rt;
use rt::entry;
use rt::exception;
use rt::ExceptionFrame;

extern crate cortex_m as cm;

extern crate cortex_m_semihosting as sh;
use sh::hio;

extern crate panic_semihosting;

#[macro_use(interrupt)]
extern crate stm32f0;
use stm32f0::stm32f0x0;
use stm32f0::stm32f0x0::exti;
use stm32f0::stm32f0x0::gpioa;

use core::fmt::Write;

struct ExtiRegWrapper(&'static exti::RegisterBlock);
unsafe impl Sync for ExtiRegWrapper {}

struct GpioaRegWrapper(&'static gpioa::RegisterBlock);
unsafe impl Sync for GpioaRegWrapper {}

static mut EXTI: Option<ExtiRegWrapper> = None;
static mut GPIO: Option<GpioaRegWrapper> = None;

#[entry]
fn main() -> ! {
    let mut core_periph = cm::peripheral::Peripherals::take().unwrap();
    let soc_periph = stm32f0x0::Peripherals::take().unwrap();
    let mut stdout = hio::hstdout().unwrap();
    let led = &soc_periph.GPIOA;

    unsafe {
        GPIO = Some(GpioaRegWrapper(&*stm32f0::stm32f0x0::GPIOA::ptr()));
        EXTI = Some(ExtiRegWrapper(&*stm32f0::stm32f0x0::EXTI::ptr()));
    }

    setup_rcc(&soc_periph);
    setup_gpio(&soc_periph);
    setup_interrupts(&mut core_periph, &soc_periph);

    loop {
        led.odr.modify(|r, w| w.odr5().bit(!r.odr5().bit()));
        writeln!(stdout, "OK").unwrap();
        delay(10000);
    }
}

fn setup_rcc(p: &stm32f0x0::Peripherals) {
    // enable GPIOA peripheral clock
    p.RCC.ahbenr.modify(|_, w| w.iopaen().set_bit());

    // enable GPIOC peripheral clock
    p.RCC.ahbenr.modify(|_, w| w.iopcen().set_bit());

    // enable SYSCFG peripheral clock
    p.RCC.apb2enr.modify(|_, w| w.syscfgen().set_bit());
}

fn setup_gpio(p: &stm32f0x0::Peripherals) {
    // configure PA5 as output pin
    p.GPIOA.moder.modify(|_, w| w.moder5().output());

    // configure PA5 pin as pull-down
    p.GPIOA.pupdr.modify(|_, w| w.pupdr5().pull_down());

    // configure PC13 as input pin
    p.GPIOC.moder.modify(|_, w| w.moder13().input());

    // configure PC13 pin as pull-down
    p.GPIOC.pupdr.modify(|_, w| w.pupdr13().pull_up());
}

fn setup_interrupts(p_core: &mut cm::peripheral::Peripherals, p_soc: &stm32f0x0::Peripherals) {
    let nvic = &mut p_core.NVIC;

    // Enable external interrupt EXTI13 for PC13
    p_soc
        .SYSCFG
        .exticr4
        .modify(|_, w| unsafe { w.exti13().bits(2) });

    // Set interrupt request mask for line 1
    p_soc.EXTI.imr.modify(|_, w| w.mr13().set_bit());

    // Set interrupt rising and falling trigger for line 13
    p_soc.EXTI.rtsr.modify(|_, w| w.tr13().set_bit());
    p_soc.EXTI.ftsr.modify(|_, w| w.tr13().clear_bit());

    // Enable EXTI IRQ, set prio 1 and clear any pending IRQs
    nvic.enable(stm32f0x0::Interrupt::EXTI4_15);
    unsafe {
        nvic.set_priority(stm32f0x0::Interrupt::EXTI4_15, 1);
    }

    cm::peripheral::NVIC::unpend(stm32f0x0::Interrupt::EXTI4_15);
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

interrupt!(EXTI4_15, button);
fn button() {
    unsafe {
        if let Some(ref gpioa) = GPIO {
            gpioa.0.odr.modify(|r, w| w.odr5().bit(!r.odr5().bit()));
        }

        if let Some(ref exti) = EXTI {
            exti.0.pr.modify(|_, w| w.pr13().set_bit());
        }
    }
}
