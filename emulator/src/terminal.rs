extern crate termios;

use std::io;
use std::io::Read;
use std::io::Write;
use termios::{Termios, TCSANOW, ECHO, ICANON, VMIN, tcsetattr};

pub struct Terminal {
    termios: Termios,
    reader: std::io::Stdin,
    stdout: std::io::Stdout,
}

impl Terminal {
    pub fn make() -> Self {
        let stdin = 0;

        let termios = Termios::from_fd(stdin).unwrap();
        let mut new_termios = termios.clone();

        new_termios.c_cc[VMIN] = 0;

        new_termios.c_lflag &= !(ICANON | ECHO);
        tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();
        let stdout = io::stdout();
        let reader = io::stdin();

        Self {termios, reader, stdout}
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        tcsetattr(0, TCSANOW, &self.termios).unwrap();

        println!("Dropping!")
    }
}

impl Read for Terminal {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Write for Terminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.lock().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.lock().flush()
    }
}