/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::convert::TryInto;
use std::ffi::c_void;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::raw::c_int;

use libc::{fflush, fread, fseek, fwrite, FILE};

use crate::libcinnabar::GetRawFd;

pub struct File(*mut FILE);

impl File {
    pub fn new(f: *mut FILE) -> Self {
        File(f)
    }

    pub unsafe fn raw(&mut self) -> *mut FILE {
        self.0
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(unsafe { fread(buf.as_mut_ptr() as *mut c_void, 1, buf.len(), self.0) })
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        if let SeekFrom::Start(pos) = pos {
            if unsafe { fseek(self.0, pos.try_into().unwrap(), libc::SEEK_SET) } < 0 {
                Err(io::Error::new(io::ErrorKind::Other, "seek error"))
            } else {
                Ok(pos)
            }
        } else {
            unimplemented!()
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(unsafe { fwrite(buf.as_ptr() as *const c_void, 1, buf.len(), self.0) })
    }

    fn flush(&mut self) -> io::Result<()> {
        unsafe {
            fflush(self.0);
        }
        Ok(())
    }
}

impl GetRawFd for File {
    fn get_writer_fd(&mut self) -> c_int {
        unsafe { libc::fileno(self.0) }
    }
}

pub struct FdFile(c_int);

impl FdFile {
    pub unsafe fn from_raw_fd(fd: c_int) -> Self {
        FdFile(fd)
    }

    pub unsafe fn raw(&mut self) -> c_int {
        self.0
    }
}

extern "C" {
    fn xread(fd: c_int, buf: *mut c_void, size: usize) -> isize;

    fn xwrite(fd: c_int, buf: *const c_void, size: usize) -> isize;
}

impl Read for FdFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            match xread(self.0, buf.as_mut_ptr() as _, buf.len()) {
                s if s < 0 => Err(io::Error::new(io::ErrorKind::Other, "read error")),
                s => Ok(s as usize),
            }
        }
    }
}

impl Write for FdFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            match xwrite(self.0, buf.as_ptr() as _, buf.len()) {
                s if s < 0 => Err(io::Error::new(io::ErrorKind::Other, "write error")),
                s => Ok(s as usize),
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}