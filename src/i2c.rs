use std::io::{Read, Write};
use std::marker::PhantomData;

use super::bitbang::*;
use super::delays::*;

pub struct I2C;

pub enum Ack {
    Ack,
    Nack,
}

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
        println!("i2c start");
        self.send_byte(0x2);
    }
    pub fn stop_bit(&mut self) {
        println!("i2c stop");
        self.send_byte(0x3);
    }
    pub fn ack(&mut self) {
        println!("i2c ack");
        self.send_byte(0x6);
    }
    pub fn nack(&mut self) {
        println!("i2c nack");
        self.send_byte(0x7);
    }
    pub fn configure(&mut self, config: Config<I2C>) {
        self.send_byte(0x40 | (0x0f & config.to_byte()));
    }

    pub fn bulk_write(&mut self, to_write: &[u8]) -> Vec<Ack> {
        flush_buffer(&mut self.port);
        let mut buf = [0u8; 17];
        let mut acks = Vec::new();
        for chunk in to_write.chunks(16) {
            assert!(chunk.len() <= 16);
            let wlen = chunk.len() + 1;

            buf[0] = 0x10 | ((chunk.len() as u8) - 1);
            buf[1..wlen].clone_from_slice(&chunk);

            self.port.write(&buf[..wlen]).unwrap();
            let mut rbuf = [0u8; 17];
            let rlen = self.port.read(&mut rbuf).unwrap();
            if rlen != wlen && !rbuf.iter().all(|x| *x == 0x01 || *x == 0x00) {
                panic!("invalid resp from i2c bulk write {:?} resp {:?}", &buf[0..wlen], &rbuf[0..rlen]);
            }
            acks.extend(rbuf[1..rlen].iter().map(|x| if *x == 0x00 { Ack::Nack } else { Ack::Ack }));
        }
        acks
    }

    pub fn read_byte(&mut self) -> u8 {
        flush_buffer(&mut self.port);
        let mut buf = [0u8; 16];
        self.port.write(&[0x4]).unwrap();
        let rlen = self.port.read(&mut buf).unwrap();
        if rlen != 1 {
            panic!("too many bytes back from i2c read: {:?}", &buf[0..rlen]);
        }
        buf[0]
    }

    pub fn write_then_read(&mut self, to_write: &[u8], read_len: usize) -> Vec<u8> {
        if to_write.len() > 4096 {
            let _ = self.write_then_read(&to_write[..4096], 0);
            return self.write_then_read(&to_write[4096..], read_len);
        }
        if read_len > 4096 {
            let mut ret = self.write_then_read(to_write, 4096);
            ret.extend_from_slice(&self.write_then_read(&[], read_len - 4096));
            return ret;
        }

        flush_buffer(&mut self.port);

        self.send_byte(0x08);

        let high_write = (to_write.len() >> 8) as u8;
        let low_write = (to_write.len() & 0x0ff) as u8;
        self.send_byte(high_write);
        self.send_byte(low_write);

        let high_read = (read_len >> 8) as u8;
        let low_read = (read_len & 0x0ff) as u8;
        self.send_byte(high_read);
        self.send_byte(low_read);

        self.port.write(to_write).unwrap();
        let mut cmd_check = [0x00];
        let mut ret = vec![0u8; read_len];

        let rlen_cmd = self.port.read(&mut cmd_check).unwrap();
        let rlen_ret = self.port.read(ret.as_mut_slice()).unwrap();
        if rlen_cmd != 1 || cmd_check[0] != 0x01  || rlen_ret != read_len {
            panic!("i2c write then read command failed, resp: {:?}", &ret);
        }
        ret
    }

    // helper functions
}

impl <'a> Eeprom for Mode<'a, I2C> {
    fn read_eeprom(&mut self, addr: u8, len: usize) -> Vec<u8> {
        self.start_bit();
        self.bulk_write(&[addr << 1, 0x00]);
        self.start_bit();
        let mut ret = self.write_then_read(&[(addr << 1) | 1], len);
        self.stop_bit();
        ret
    }

    fn write_eeprom(&mut self, addr: u8, to_write: &[u8], page_sz: usize) {
        unimplemented!()
    }
}
