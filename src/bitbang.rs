use std::ops::{Drop, Deref, DerefMut};
use std::io::{Read, Write};

use std::marker::PhantomData;

use serial::SystemPort;
use serial::prelude::*;

use super::delays::*;

pub type DeviceAddress = Vec<u8>;

pub struct BitBangMode {
    pub port: SystemPort,
}
pub struct Mode<'a, T> {
    pub port: &'a mut BitBangMode,
    pub _ty: PhantomData<T>,
}

#[derive(Clone, Copy)]
pub struct Config<T> {
    pub bits: u8,
    _ty: PhantomData<T>,
}

pub trait Eeprom {
    fn read_eeprom(&mut self, addr: u8, len: usize) -> Vec<u8>;
    fn write_eeprom(&mut self, addr: u8, to_write: &[u8], page_sz: usize);
}

impl Deref for BitBangMode {
    type Target = SystemPort;
    fn deref(&self) -> &SystemPort {
        &self.port
    }
}
impl DerefMut for BitBangMode {
    fn deref_mut(&mut self) -> &mut SystemPort {
        &mut self.port
    }
}

impl Drop for BitBangMode {
    fn drop(&mut self) {
        reset_buspirate(&mut self.port);
    }
}

impl <'a, T> Drop for Mode<'a, T> {
    fn drop(&mut self) {
        reset_to_bitbang(&mut self.port);
    }
}

impl <T> Config<T> {
    pub fn new() -> Config<T> {
        Config {
            bits: 0,
            _ty: PhantomData,
        }
    }
    pub fn power_on(self) -> Config<T> {
        Config { bits: self.bits | 0x08, _ty: PhantomData, }
    }
    pub fn power_off(self) -> Config<T> {
        Config { bits: self.bits & !0x08, _ty: PhantomData, }
    }
    pub fn pullup_on(self) -> Config<T> {
        Config { bits: self.bits | 0x04, _ty: PhantomData, }
    }
    pub fn pullup_off(self) -> Config<T> {
        Config { bits: self.bits & !0x04, _ty: PhantomData, }
    }
    pub fn aux_on(self) -> Config<T> {
        Config { bits: self.bits | 0x02, _ty: PhantomData, }
    }
    pub fn aux_off(self) -> Config<T> {
        Config { bits: self.bits & !0x02, _ty: PhantomData, }
    }
    pub fn cs_on(self) -> Config<T> {
        Config { bits: self.bits | 0x01, _ty: PhantomData, }
    }
    pub fn cs_off(self) -> Config<T> {
        Config { bits: self.bits & !0x01, _ty: PhantomData, }
    }

    pub fn to_byte(self) -> u8 {
        self.bits
    }
}

pub fn flush_buffer(port: &mut SystemPort) {
    let mut buf = [0u8; 16];
    let mut rlen = port.read(&mut buf).ok().unwrap_or_else(|| 0);
    println!("buffer flushed with {:?}", &buf[0..rlen]);
    while rlen > 0 {
        println!("buffer flushed with {:?}", &buf[0..rlen]);
        rlen = port.read(&mut buf).ok().unwrap_or_else(|| 0);
    }
}

pub fn enter_bitbang_mode(mut port: SystemPort) -> BitBangMode {
    let mut buf = [0u8; 16];
    let mut rlen = 0;
    let mut count = 0;
    // empty the buffer
    flush_buffer(&mut port);

    while &buf[0..rlen] != b"BBIO1" && count < 25 {
        port.write(&[0x00]).unwrap();
        sleep(RESP_DELAY_REALLY_SHORT.clone());
        rlen = port.read(&mut buf).ok().unwrap_or_else(|| 0);
        count += 1;

        println!("bitbang try {}, resp {:?}", count, &buf[0..rlen]);
    }
    println!("bitbang mode detected!");
    BitBangMode { port: port }
}

pub fn poweron(port: &mut SystemPort) -> u8 {
    let mut buf = [0u8; 16];

    flush_buffer(port);
    port.write(&[0xc0]).unwrap();
    sleep(RESP_DELAY_SHORT.clone());
    let len = port.read(&mut buf).ok().unwrap_or_else(|| 0);
    println!("poweron resp {:?}", &buf[0..len]);
    if len < 1 {
        panic!("invalid response");
    }
    buf[0]
}

fn reset_to_bitbang(port: &mut SystemPort) {
    flush_buffer(port);
    println!("reseting to bitbang..");
    port.write(&[0x00]).unwrap();
    sleep(RESP_DELAY_LONG.clone());
    let mut buf = [0u8; 16];
    let len = port.read(&mut buf).ok().unwrap_or_else(|| 0);
    println!("port reset resp {:?}", &buf[0..len]);
    if len < 1 || &buf[0..len] != b"BBIO1" {
        panic!("invalid response");
    }
}

fn reset_buspirate(port: &mut SystemPort) {
    flush_buffer(port);
    println!("reseting bus pirate..");
    port.write(&[0x0f]).unwrap();
    sleep(RESP_DELAY_LONG.clone());
    let mut buf = [0u8; 16];
    let len = port.read(&mut buf).ok().unwrap_or_else(|| 0);
    println!("reset resp {:?}", &buf[0..len]);
    if len < 1 || buf[0] != 0x01 {
        panic!("invalid response");
    }

}


