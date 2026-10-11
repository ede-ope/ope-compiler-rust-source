#![no_std]
use core::panic::PanicInfo;

#[cfg(target_os = "windows")]
mod startup {
    unsafe extern "system" {
        fn ExitProcess(code: u32) -> !;
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn __kui_runtime_init_inner() {
        unsafe {
            SetConsoleCP(65001);
            SetConsoleOutputCP(65001);
        }
    }

    unsafe extern "system" {
        fn SetConsoleCP(wCodePageID: u32) -> i32;
        fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    }
}

#[cfg(target_os = "linux")]
mod startup {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn __kui_runtime_init_inner() {
        // just leave empty for now
    }
}

#[cfg(target_os = "macos")]
mod startup {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn __kui_runtime_init_inner() {
        // macOS init
    }
}

// Keep the no_std panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
