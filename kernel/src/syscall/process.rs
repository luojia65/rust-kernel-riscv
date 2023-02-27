//! The `process` module provides system calls to interact with processes.

use log::info;

use crate::batch;

/// Exit the current process with an exit code.
pub fn sys_exit(exit_code: i32) -> ! {
    info!("exited with {}", exit_code);
    batch::load_next_bin();
}