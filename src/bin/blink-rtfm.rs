#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m as cm;
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;

extern crate stm32f0;
use stm32f0::stm32f0x0;

extern crate rtfm;
use rtfm::app;

#[app(device = stm32f0x0)]
const APP: () = {
    // resources
    static tim3: stm32f0x0::TIM3 = ();
    static tim14: stm32f0x0::TIM14 = ();
    static gpioa: stm32f0x0::GPIOA = ();

    #[init]
    fn init() {
        // enable GPIOA peripheral clock
        device.RCC.ahbenr.modify(|_, w| w.iopaen().set_bit());

        // enable TIM14 peripheral clock
        device.RCC.apb1enr.modify(|_, w| w.tim14en().set_bit());

        // enable TIM3 peripheral clock
        device.RCC.apb1enr.modify(|_, w| w.tim3en().set_bit());

        // configure PA5 as output pin
        device.GPIOA.moder.modify(|_, w| w.moder5().output());

        // configure PA5 pin as pull-down
        device.GPIOA.pupdr.modify(|_, w| w.pupdr5().pull_down());

        // TIM14 configuration

        unsafe {
            // set timer start value
            device.TIM14.cnt.modify(|_, w| w.bits(1));

            // set timer prescaler: 8MHz/800 => 10000 ticks per second
            device.TIM14.psc.modify(|_, w| w.bits(800));

            // set timer value when interrupt is generated: once per 3 seconds
            device.TIM14.arr.modify(|_, w| w.bits(30000));
        }

        // set timer value when interrupt is generated: once per second
        device.TIM14.dier.modify(|_, w| w.uie().set_bit());

        // TIM3 configuration

        unsafe {
            // set timer start value
            device.TIM3.cnt.modify(|_, w| w.bits(1));

            // set timer prescaler: 8MHz/800 => 10000 ticks per second
            device.TIM3.psc.modify(|_, w| w.bits(800));

            // set timer value when interrupt is generated: once per second
            device.TIM3.arr.modify(|_, w| w.bits(10000));
        }

        // set timer value when interrupt is generated: once per second
        device.TIM3.dier.modify(|_, w| w.uie().set_bit());

        // start timers
        device.TIM14.cr1.modify(|_, w| w.cen().set_bit());
        device.TIM3.cr1.modify(|_, w| w.cen().set_bit());

        // init late resources
        tim14 = device.TIM14;
        tim3 = device.TIM3;
        gpioa = device.GPIOA;
    }

    #[idle]
    fn idle() -> ! {
        loop {
            cm::asm::wfi();
        }
    }

    #[interrupt(resources = [tim3, gpioa])]
    fn TIM3() {
        (*resources.gpioa)
            .odr
            .modify(|r, w| w.odr5().bit(!r.odr5().bit()));

        (*resources.tim3).sr.modify(|_, w| w.uif().clear_bit());
    }

    #[interrupt(resources = [tim14])]
    fn TIM14() {
        (*resources.tim14).sr.modify(|_, w| w.uif().clear_bit());
    }
};
