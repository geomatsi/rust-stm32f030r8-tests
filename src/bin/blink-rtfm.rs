#![no_main]
#![no_std]

extern crate cortex_m as cm;
extern crate cortex_m_rt as rt;

extern crate cortex_m_rtfm as rtfm;
use rtfm::{app, Threshold};

extern crate cortex_m_semihosting as sh;
use sh::hio;

extern crate panic_semihosting;

extern crate stm32f0;
use stm32f0::stm32f0x0;

use core::fmt::Write;

app! {
    device: stm32f0x0,

    resources: {
        static tim3: stm32f0x0::TIM3;
        static tim14: stm32f0x0::TIM14;
        static gpioa: stm32f0x0::GPIOA;
    },

    tasks: {
        TIM3: {
            priority: 1,
            path: tim3_handler,
            resources: [tim3, gpioa],
        },
        TIM14: {
            priority: 1,
            path: tim14_handler,
            resources: [tim14],
        }
    }
}

fn init(p: init::Peripherals) -> init::LateResources {
    // enable GPIOA peripheral clock
    p.device.RCC.ahbenr.modify(|_, w| w.iopaen().set_bit());

    // enable TIM14 peripheral clock
    p.device.RCC.apb1enr.modify(|_, w| w.tim14en().set_bit());

    // enable TIM3 peripheral clock
    p.device.RCC.apb1enr.modify(|_, w| w.tim3en().set_bit());

    // configure PA5 as output pin
    p.device.GPIOA.moder.modify(|_, w| w.moder5().output());

    // configure PA5 pin as pull-down
    p.device.GPIOA.pupdr.modify(|_, w| w.pupdr5().pull_down());

    // TIM14 configuration

    unsafe {
        // set timer start value
        p.device.TIM14.cnt.modify(|_, w| w.bits(1));

        // set timer prescaler: 8MHz/800 => 10000 ticks per second
        p.device.TIM14.psc.modify(|_, w| w.bits(800));

        // set timer value when interrupt is generated: once per 3 seconds
        p.device.TIM14.arr.modify(|_, w| w.bits(30000));
    }

    // set timer value when interrupt is generated: once per second
    p.device.TIM14.dier.modify(|_, w| w.uie().set_bit());

    // TIM3 configuration

    unsafe {
        // set timer start value
        p.device.TIM3.cnt.modify(|_, w| w.bits(1));

        // set timer prescaler: 8MHz/800 => 10000 ticks per second
        p.device.TIM3.psc.modify(|_, w| w.bits(800));

        // set timer value when interrupt is generated: once per second
        p.device.TIM3.arr.modify(|_, w| w.bits(10000));
    }

    // set timer value when interrupt is generated: once per second
    p.device.TIM3.dier.modify(|_, w| w.uie().set_bit());

    // start timers
    p.device.TIM14.cr1.modify(|_, w| w.cen().set_bit());
    p.device.TIM3.cr1.modify(|_, w| w.cen().set_bit());

    // init late resources
    init::LateResources {
        tim14: p.device.TIM14,
        tim3: p.device.TIM3,
        gpioa: p.device.GPIOA,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn tim3_handler(_t: &mut Threshold, r: TIM3::Resources) {
    (*r.gpioa).odr.modify(|r, w| w.odr5().bit(!r.odr5().bit()));

    (*r.tim3).sr.modify(|_, w| w.uif().clear_bit());
}

fn tim14_handler(_t: &mut Threshold, r: TIM14::Resources) {
    let mut stdout = hio::hstdout().unwrap();

    writeln!(stdout, "TIM14").unwrap();

    (*r.tim14).sr.modify(|_, w| w.uif().clear_bit());
}
