#![no_std]

/// tracking of state via global variables
static mut OLD_CS: bool = false;
static mut CS: bool = false;

/// Execute closure `f` in an interrupt-free context.
///
/// This as also known as a "critical section".
///
#[inline]
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        core::arch::asm!("cpsid i");
    }
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    // critical section begin
    unsafe { OLD_CS = CS };
    unsafe { CS = true };

    let r = f();

    if unsafe { !OLD_CS } {
        unsafe { OLD_CS = false };
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        // critical section end
        unsafe {
            core::arch::asm!("cpsie i");
        }
    }
    r
}
