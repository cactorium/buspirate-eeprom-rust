use std::io::{Read, Write};
use std::marker::PhantomData;

use super::bitbang::*;
use super::delays::*;

pub struct I2C;

impl <'a> Mode<'a, I2C> {
    pub fn i2c(port: &'a mut BitBangMode) -> Mode<'a, I2C> {
        let mut buf = [0u8; 16];

        port.write(&[0x02]).unwrap();
        sleep(RESP_DELAY_SHORT.clone());
        
        let rlen = port.read(&mut buf).unwrap();
        println!("i2c mode try, resp {:?}", &buf[0..rlen]);

        if &buf[0..rlen] != b"I2C1" {
            panic!("i2c mode entry failed!");
        }
        println!("i2c mode entered!");
        Mode { port: port, _ty: PhantomData }
    }

    fn send_byte(&mut self, b: u8) {
        let mut buf = [0u8; 16];
        self.port.write(&[b]).unwrap();
        let rlen = self.port.read(&mut buf).unwrap();
        if rlen != 1 || buf[0] != 0x01 {
            panic!("start bit send failed!");
        }
    }

    pub fn start_bit(&mut self) {
        self.send_byte(0x2);
    }
    pub fn stop_bit(&mut self) {
        self.send_byte(0x3);
    }
    pub fn ack(&mut self) {
        self.send_byte(0x6);
    }
    pub fn nack(&mut self) {
        self.send_byte(0x7);
    }

    pub fn send_cmd_then_read(&mut self, to_write: &[u8], read_len: usize) -> Vec<u8> {
        unimplemented!()
    }
}


