#![feature(prelude_import)]
//! examples/hello_world.rs
#![deny(unsafe_code)]
#![no_main]
#![no_std]
#[macro_use]
extern crate core;
#[prelude_import]
use core::prelude::rust_2021::*;
pub mod app {
    /// Include peripheral crate(s) that defines the vector table
    use nrf52840_hal::pac as _;
    use cortex_m::asm;
    use nrf52840_hal::{self as _, pac};
    use panic_halt as _;
    /// Module defining rtic traits
    pub use rtic_traits::*;
    pub mod rtic_traits {
        /// Trait for a hardware task
        pub trait RticTask {
            /// Associated type that can be used to make [Self::init] take arguments
            type InitArgs: Sized;
            /// Task local variables initialization routine
            fn init(args: Self::InitArgs) -> Self;
            /// Function to be bound to a HW Interrupt
            fn exec(&mut self);
        }
        /// Trait for an idle task
        pub trait RticIdleTask {
            /// Associated type that can be used to make [Self::init] take arguments
            type InitArgs: Sized;
            /// Task local variables initialization routine
            fn init(args: Self::InitArgs) -> Self;
            /// Function to be executing when no other task is running
            fn exec(&mut self) -> !;
        }
        pub trait RticMutex {
            type ResourceType;
            fn lock(&mut self, f: impl FnOnce(&mut Self::ResourceType));
        }
    }
    /// critical section function
    #[inline]
    pub fn __rtic_interrupt_free<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        unsafe {
            asm!("cpsid i");
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        unsafe { OLD_CS = CS };
        unsafe { CS = true };
        let r = f();
        if unsafe { !OLD_CS } {
            unsafe { OLD_CS = false };
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
            unsafe {
                asm!("cpsie i");
            }
        }
        r
    }
    /// Software tasks of
    /// Core 0
    /// Dispatchers of
    /// Core 0
    /// RTIC Software task trait
    /// Trait for a software task
    pub trait RticSwTask {
        type InitArgs: Sized;
        type SpawnInput;
        /// Task local variables initialization routine
        fn init(args: Self::InitArgs) -> Self;
        /// Function to be executing when the scheduled software task is dispatched
        fn exec(&mut self, input: Self::SpawnInput);
    }
    /// Core local interrupt pending
    #[doc(hidden)]
    #[inline]
    pub fn __rtic_local_irq_pend<I: rtic::export::InterruptNumber>(irq_nbr: I) {
        rtic::export::NVIC::pend(irq_nbr);
    }
    /// # CORE 0
    static mut SHARED_RESOURCES: core::mem::MaybeUninit<SharedResources> = core::mem::MaybeUninit::uninit();
    struct SharedResources {
        x: u64,
    }
    fn system_init() -> SharedResources {
        SharedResources { x: 0 }
    }
    static mut MY_IDLE_TASK: core::mem::MaybeUninit<MyIdleTask> = core::mem::MaybeUninit::uninit();
    pub struct MyIdleTask {
        count: u32,
    }
    const _: fn() = || {
        __rtic_trait_checks::implements_rtic_idle_task::<MyIdleTask>();
    };
    impl RticIdleTask for MyIdleTask {
        fn init(_: ()) -> Self {
            Self { count: 0 }
        }
        fn exec(&mut self) -> ! {
            self.shared().x.lock(|_| {});
            loop {
                self.count += 1;
                rtic::export::NVIC::pend(pac::Interrupt::TIMER0);
                asm::delay(48_000_000);
            }
        }
        type InitArgs = ();
    }
    impl MyIdleTask {
        pub const fn priority() -> u16 {
            1u16
        }
    }
    impl MyIdleTask {
        pub fn shared(&self) -> __my_idle_task_shared_resources {
            const TASK_PRIORITY: u16 = 1u16;
            __my_idle_task_shared_resources::new(TASK_PRIORITY)
        }
    }
    pub struct __my_idle_task_shared_resources {
        pub x: __x_mutex,
    }
    impl __my_idle_task_shared_resources {
        #[inline(always)]
        pub fn new(priority: u16) -> Self {
            Self {
                x: __x_mutex::new(priority),
            }
        }
    }
    impl MyIdleTask {
        pub const fn current_core() -> __rtic__internal__Core0 {
            unsafe { __rtic__internal__Core0::new() }
        }
    }
    static mut T0: core::mem::MaybeUninit<T0> = core::mem::MaybeUninit::uninit();
    pub struct T0 {
        count: u32,
    }
    const _: fn() = || {
        __rtic_trait_checks::implements_rtic_task::<T0>();
    };
    impl RticTask for T0 {
        fn init(_: ()) -> Self {
            Self { count: 0 }
        }
        fn exec(&mut self) {
            self.shared()
                .x
                .lock(|_| {
                    self.count += 1;
                    rtic::export::NVIC::pend(pac::Interrupt::TIMER1);
                });
        }
        type InitArgs = ();
    }
    impl T0 {
        pub const fn priority() -> u16 {
            7u16
        }
    }
    impl T0 {
        pub fn shared(&self) -> __t0_shared_resources {
            const TASK_PRIORITY: u16 = 7u16;
            __t0_shared_resources::new(TASK_PRIORITY)
        }
    }
    pub struct __t0_shared_resources {
        pub x: __x_mutex,
    }
    impl __t0_shared_resources {
        #[inline(always)]
        pub fn new(priority: u16) -> Self {
            Self {
                x: __x_mutex::new(priority),
            }
        }
    }
    impl T0 {
        pub const fn current_core() -> __rtic__internal__Core0 {
            unsafe { __rtic__internal__Core0::new() }
        }
    }
    #[allow(non_snake_case)]
    #[no_mangle]
    fn TIMER0() {
        rtic::export::run(
            7u16 as u8,
            || {
                unsafe { T0.assume_init_mut().exec() };
            },
        );
    }
    pub struct __x_mutex {
        #[doc(hidden)]
        task_priority: u16,
    }
    impl __x_mutex {
        #[inline(always)]
        pub fn new(task_priority: u16) -> Self {
            Self { task_priority }
        }
    }
    impl RticMutex for __x_mutex {
        type ResourceType = u64;
        fn lock(&mut self, f: impl FnOnce(&mut Self::ResourceType)) {
            const CEILING: u16 = 7u16;
            let task_priority = self.task_priority;
            let resource_ptr = unsafe { &mut SHARED_RESOURCES.assume_init_mut().x }
                as *mut _;
            unsafe {
                rtic::export::lock(resource_ptr, CEILING as u8, NVIC_PRIO_BITS, f);
            }
        }
    }
    ///Unique type for core 0
    pub use core0_type_mod::__rtic__internal__Core0;
    mod core0_type_mod {
        struct __rtic__internal__Core0Inner;
        pub struct __rtic__internal__Core0(__rtic__internal__Core0Inner);
        impl __rtic__internal__Core0 {
            pub const unsafe fn new() -> Self {
                __rtic__internal__Core0(__rtic__internal__Core0Inner)
            }
        }
    }
    static mut OLD_CS: bool = false;
    static mut CS: bool = false;
    use nrf52840_hal::pac::NVIC_PRIO_BITS;
    /// Type representing tasks that need explicit user initialization
    /// Entry of
    /// # CORE 0
    #[no_mangle]
    fn main() -> ! {
        __rtic_interrupt_free(|| {
            let shared_resources = system_init();
            unsafe {
                SHARED_RESOURCES.write(shared_resources);
            }
            unsafe {
                T0.write(T0::init(()));
            }
            unsafe {
                if !(0 < 7u16 && 7u16 <= 1 << NVIC_PRIO_BITS) {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!("priority level not supported"),
                        );
                    }
                }
                nrf52840_hal::pac::CorePeripherals::steal()
                    .NVIC
                    .set_priority(
                        nrf52840_hal::pac::Interrupt::TIMER0,
                        rtic::export::cortex_logical2hw(7u16 as u8, NVIC_PRIO_BITS),
                    );
                nrf52840_hal::pac::NVIC::unmask(nrf52840_hal::pac::Interrupt::TIMER0);
            }
        });
        unsafe {
            MY_IDLE_TASK.write(MyIdleTask::init(()));
            MY_IDLE_TASK.assume_init_mut().exec();
        }
    }
    /// Utility functions used to enforce implementing appropriate task traits
    mod __rtic_trait_checks {
        use super::*;
        pub fn implements_rtic_task<T: RticTask>() {}
        pub fn implements_rtic_idle_task<T: RticIdleTask>() {}
    }
}
