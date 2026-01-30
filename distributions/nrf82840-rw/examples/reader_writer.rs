//! examples/hello_world.rs

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

#[rtic::app(device = nrf52840_hal::pac)]
mod app {
    use cortex_m::asm;

    use defmt::*;
    use defmt_rtt as _;
    use nrf52840_hal::pac;
    use panic_probe as _;

    #[shared]
    struct SharedResources {
        x: u64,
    }

    #[init]
    fn system_init() -> SharedResources {
        for i in 0..10 {
            println!("hello_world #{}", i);
        }

        SharedResources { x: 0 }
    }

    #[idle(shared = [x])]
    pub struct MyIdleTask {
        count: u32,
    }
    impl RticIdleTask for MyIdleTask {
        fn init() -> Self {
            Self { count: 0 }
        }

        fn exec(&mut self) -> ! {
            self.shared().x.lock(|_| {});
            loop {
                self.count += 1;
                println!("looping in idle... {}", self.count);
                rtic::export::NVIC::pend(pac::Interrupt::TIMER0);
                asm::delay(48_000_000);
            }
        }
    }

    #[task(deadline = 100, binds = TIMER0, shared = [x])]
    pub struct T0 {
        count: u32,
    }
    impl RticTask for T0 {
        fn init() -> Self {
            Self { count: 0 }
        }

        fn exec(&mut self) {
            self.shared().x.lock(|_| {
                self.count += 1;
                println!("Timer0... {}", self.count);
                rtic::export::NVIC::pend(pac::Interrupt::TIMER1);
                println!("Timer0... {}", self.count);
            });
            println!("Timer0");
        }
    }

    #[task(deadline = 50, binds = TIMER1, shared = [x])]
    pub struct T2 {
        count: u32,
    }
    impl RticTask for T2 {
        fn init() -> Self {
            Self { count: 0 }
        }

        fn exec(&mut self) {
            self.shared().x.lock(|_| {});
            self.count += 2;
            println!("Timer1... {}", self.count);
            if self.count == 6 {
                self::panic!("--done--");
            }
        }
    }
}
