//! Module providing some convience functions for using termios
use crate::serialport::{Error, Result};

use nix::sys::termios::{self, tcgetattr, tcsetattr, SetArg, Termios};
use std::os::unix::io::RawFd;

pub fn get_termios(fd: &RawFd) -> Result<Termios> {
    let termios = match tcgetattr(*fd) {
        Ok(t) => t,
        Err(e) => return Err(e.into()),
    };

    Ok(termios)
}

pub fn set_termios(fd: &mut RawFd, termios: &Termios) -> Result<()> {
    match tcsetattr(*fd, SetArg::TCSANOW, termios) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
