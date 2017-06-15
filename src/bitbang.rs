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

pub fn clear_buf(port: &mut SystemPort) {
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
    clear_buf(&mut port);

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

    clear_buf(port);
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
    clear_buf(port);
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
    clear_buf(port);
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


