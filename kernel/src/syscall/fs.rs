//! The `fs` module provides system calls to interact with the file system.

use core::str;

use log::error;

use crate::{executor::ControlFlow, mem::translate_buffer, print, syscall::SystemCall};

const STDOUT: usize = 1;

impl SystemCall<'_> {
    pub fn sys_read(&self, _fd: usize, _buffer: *const u8, _length: usize) -> (isize, ControlFlow) {
        (-1, ControlFlow::Continue)
    }

    /// Writes the contents of a buffer to a file descriptor.
    pub fn sys_write(&self, fd: usize, buffer: *const u8, length: usize) -> (isize, ControlFlow) {
        match fd {
            STDOUT => {
                for buffer in translate_buffer(self.thread.satp(), buffer, length) {
                    print!("{}", str::from_utf8(buffer).unwrap());
                }
                (length as isize, ControlFlow::Continue)
            }
            _ => {
                error!("the file descriptor {} is not supported in 'sys_write'", fd);
                (-1, ControlFlow::Continue)
            }
        }
    }
}
