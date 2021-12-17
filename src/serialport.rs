//!
use crate::log::{debug, log};
use crate::termios;
use nix::fcntl::{self, OFlag};
use nix::sys::stat::Mode;
pub use nix::sys::termios::BaudRate;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::time::Duration;

use crate::termios::{get_termios, set_termios};
use std::error::Error as stderr;
use std::fmt;

//TODO: Add the kinds of errors
#[derive(Debug, Copy, Clone)]
pub enum ErrorKind {
    Unknown,
    PortClosed,
    Errno(nix::errno::Errno),
}

#[derive(Debug)]
pub struct Error {
    /// Kind of error
    kind: ErrorKind,
    /// Long description of error
    description: String,
}

impl stderr for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        fmt.write_str(&self.description)
    }
}

impl Error {
    pub fn new(kind: ErrorKind, description: &str) -> Error {
        Error {
            kind,
            description: description.to_string(),
        }
    }
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn desc(&self) -> &String {
        &self.description
    }
}
//TODO: At some point I should update this to
//match specific errors but for now
//it will me fen to just wrap Errno in my enum
impl From<nix::errno::Errno> for Error {
    fn from(e: nix::errno::Errno) -> Error {
        Error::new(ErrorKind::Errno(e), e.desc())
    }
}

pub struct SerialPort {
    fd: Option<RawFd>,
    path: String,
    baud: BaudRate,
    timeout: Duration,
}
pub type Result<T> = std::result::Result<T, Error>;

impl Drop for SerialPort {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl SerialPort {
    pub fn new(path: &str, baud: BaudRate, timeout: Duration) -> Result<SerialPort> {
        Ok(SerialPort {
            path: path.into(),
            fd: None,
            baud,
            timeout,
        })
    }

    /// Write bytes from arr to open serial port
    pub fn write(&self, arr: &[u8]) -> Result<usize> {
        use nix::unistd::write;
        match self.fd {
            Some(fd) => match write(fd, arr) {
                Ok(n) => Ok(n),
                Err(e) => Err(e.into()),
            },
            None => Err(Error::new(ErrorKind::PortClosed, "Serial port is not open")),
        }
    }
    /// Read bytes from the serial port into
    /// the the supplied array
    pub fn read(&self, arr: &mut [u8]) -> Result<usize> {
        use nix::unistd::read;
        match self.fd {
            Some(fd) => match read(fd, arr) {
                Ok(n) => Ok(n),
                Err(e) => Err(e.into()),
            },
            None => Err(Error::new(ErrorKind::PortClosed, "Serial port is not open")),
        }
    }

    /// Close the serial port
    pub fn close(&mut self) -> Result<()> {
        use nix::unistd::close;
        match self.fd {
            Some(fd) => match close(fd) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            },
            None => Err(Error::new(ErrorKind::PortClosed, "Serial port is not open")),
        }
    }
    pub fn flush(&self) -> Result<()> {
        use nix::sys::termios::{tcflush, FlushArg};
        match self.fd {
            Some(fd) => match tcflush(fd, FlushArg::TCIFLUSH) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            },
            None => Err(Error::new(ErrorKind::PortClosed, "Serial port is not open")),
        }
    }

    //TODO: Add some way to configure the port
    //before you open. Might actually implement
    //a builder pattern.
    /// Open the serial port
    pub fn open(&mut self) -> Result<()> {
        use nix::fcntl::fcntl;
        use nix::fcntl::FcntlArg::F_SETFL;
        use nix::sys::termios::{
            cfsetispeed, cfsetospeed, ControlFlags, InputFlags, LocalFlags, OutputFlags,
            SpecialCharacterIndices,
        };
        // Unwrapping for now, eventually I will
        // replace with returning my own error
        let mut fd = match fcntl::open(
            Path::new(&self.path),
            OFlag::O_NOCTTY | OFlag::O_RDWR | OFlag::O_NONBLOCK,
            Mode::empty(),
        ) {
            Ok(n) => n,
            Err(e) => {
                debug(&format!("Serial: {:?}", e));
                return Err(e.into());
            }
        };
        let mut settings = get_termios(&fd)?;

        // just set it how I want
        // until I figure out what I want to do with
        // settings
        settings.control_flags &= !ControlFlags::PARENB;
        settings.control_flags &= !ControlFlags::CSTOPB;
        settings.control_flags &= !ControlFlags::CSIZE;
        settings.control_flags |= ControlFlags::CS8;
        settings.control_flags &= !ControlFlags::CRTSCTS;
        settings.control_flags |= ControlFlags::CREAD | ControlFlags::CLOCAL;
        settings.local_flags &= !LocalFlags::ICANON;
        settings.local_flags &= !LocalFlags::ECHO;
        settings.local_flags &= !LocalFlags::ECHOE;
        settings.local_flags &= !LocalFlags::ECHONL;
        settings.local_flags &= !LocalFlags::ISIG;
        settings.input_flags &= !(InputFlags::IXON | InputFlags::IXOFF | InputFlags::IXANY);
        settings.input_flags &= !(InputFlags::IGNBRK
            | InputFlags::BRKINT
            | InputFlags::PARMRK
            | InputFlags::ISTRIP
            | InputFlags::INLCR
            | InputFlags::ICRNL);
        settings.output_flags &= !OutputFlags::OPOST;
        settings.output_flags &= !OutputFlags::ONLCR;
        //Used for timeout and read behavior
        //
        //NOTE: VTIME's units are deciseconds
        //control_chars is a &[u8] so the maximum time out using
        // VTIME is 25.5 seconds which is 255 deciseconds
        let vtime = {
            let sec = self.timeout.as_secs_f32();
            if sec > 25.5 {
                255
            } else {
                // should give me seconds
                // in deciseconds
                (sec * 10.0) as u8
            }
        };
        settings.control_chars[SpecialCharacterIndices::VTIME as usize] = vtime;
        //TODO: Maybe implement a way to set and use VMIN to control the minimim
        //number of characters
        settings.control_chars[SpecialCharacterIndices::VMIN as usize] = 1;
        cfsetospeed(&mut settings, self.baud)?;
        cfsetispeed(&mut settings, self.baud)?;
        set_termios(&mut fd, &settings)?;
        fcntl(fd, F_SETFL(nix::fcntl::OFlag::empty()))?;
        self.fd = Some(fd);
        Ok(())
    }

    ///Set the baud rate.
    ///
    ///Calling this will set the rate immediately if
    ///the port is open. Otherwise it will be set once open
    ///is called.
    fn set_baud(&mut self, baud: BaudRate) -> Result<()> {
        use nix::sys::termios::{cfsetispeed, cfsetospeed};
        // TODO: if the serial port is not open,
        // just set the rate
        // otherwise we should immediately apply the settings
        match self.fd {
            None => {
                self.baud = baud;
                Ok(())
            }
            Some(mut fd) => {
                self.baud = baud;
                let mut settings = get_termios(&fd)?;

                cfsetospeed(&mut settings, self.baud)?;
                cfsetispeed(&mut settings, self.baud)?;
                set_termios(&mut fd, &settings)?;
                Ok(())
            }
        }
    }
    /// Set the timeout
    ///
    /// Calling this will set the timeout immediately if
    /// the port is open. Otherwise, it will be set once
    /// open is called.
    fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        use nix::sys::termios::SpecialCharacterIndices;
        //TODO:
        //Same as set_baud
        match self.fd {
            None => {
                self.timeout = timeout;
                Ok(())
            }
            Some(mut fd) => {
                self.timeout = timeout;
                let mut settings = get_termios(&fd)?;
                //VTIME's units are deciseconds
                let vtime = {
                    let sec = self.timeout.as_secs_f32();
                    if sec > 25.5 {
                        255
                    } else {
                        // should give me seconds
                        // in deciseconds
                        (sec * 10.0) as u8
                    }
                };
                settings.control_chars[SpecialCharacterIndices::VTIME as usize] = vtime;
                set_termios(&mut fd, &settings)?;
                Ok(())
            }
        }
    }
}
