//! Exercise the real BK clock-start function with simulated MCU registers.
//! This checks clock selection and timeout behavior, not oscillator accuracy.
#![allow(dead_code)]
extern crate self as at32f421_pac;
use std::cell::Cell;

pub struct Crm {
    ctrl: Register,
    cfg: Register,
    pll: Register,
}
pub struct Flash {
    psr: Register,
}
impl Crm {
    fn new(ready_after: Option<u32>) -> Self {
        Self {
            ctrl: Register::new(0, 3, ready_after),
            cfg: Register::new(1, 0, None),
            pll: Register::new(2, 0x1f10, None),
        }
    }
    pub fn ctrl(&self) -> &Register {
        &self.ctrl
    }
    pub fn cfg(&self) -> &Register {
        &self.cfg
    }
    pub fn pll(&self) -> &Register {
        &self.pll
    }
}
impl Flash {
    pub fn psr(&self) -> &Register {
        &self.psr
    }
}
pub struct Register {
    kind: u8,
    value: Cell<u32>,
    ready_after: Option<u32>,
    polls: Cell<u32>,
}
impl Register {
    fn new(kind: u8, value: u32, ready_after: Option<u32>) -> Self {
        Self {
            kind,
            value: Cell::new(value),
            ready_after,
            polls: Cell::new(0),
        }
    }
    pub fn read(&self) -> Read {
        let mut value = self.value.get();
        if self.kind == 0 && value & (1 << 16) != 0 {
            let polls = self.polls.get();
            self.polls.set(polls + 1);
            if self.ready_after.is_some_and(|n| polls >= n) {
                value |= 1 << 17;
                self.value.set(value);
            }
        }
        Read(value)
    }
    fn store(&self, mut value: u32) {
        if self.kind == 0 {
            if value & 1 != 0 {
                value |= 2;
            }
            if value & (1 << 24) != 0 {
                value |= 1 << 25;
            } else {
                value &= !(1 << 25);
            }
            if value & (1 << 16) == 0 {
                value &= !(1 << 17);
            }
        }
        if self.kind == 1 {
            value = (value & !12) | ((value & 3) << 2);
        }
        self.value.set(value);
    }
    pub fn modify(
        &self,
        f: impl for<'a> FnOnce(&Read, &'a mut Write) -> &'a mut Write,
    ) {
        let r = self.read();
        let mut w = Write(r.0);
        f(&r, &mut w);
        self.store(w.0);
    }
    pub fn write(&self, f: impl FnOnce(&mut Write) -> &mut Write) {
        let mut w = Write(0);
        f(&mut w);
        self.store(w.0);
    }
}
pub struct Read(u32);
pub struct Field(u8);
impl Field {
    pub fn bits(&self) -> u8 {
        self.0
    }
    pub fn bit_is_clear(&self) -> bool {
        self.0 == 0
    }
    pub fn bit_is_set(&self) -> bool {
        self.0 != 0
    }
}
impl Read {
    pub fn sclksts(&self) -> Field {
        Field(((self.0 >> 2) & 3) as u8)
    }
    pub fn pllen(&self) -> Field {
        Field(((self.0 >> 24) & 1) as u8)
    }
    pub fn hickstbl(&self) -> Field {
        Field(((self.0 >> 1) & 1) as u8)
    }
    pub fn hextstbl(&self) -> Field {
        Field(((self.0 >> 17) & 1) as u8)
    }
    pub fn pllstbl(&self) -> Field {
        Field(((self.0 >> 25) & 1) as u8)
    }
}
pub struct Write(u32);
pub struct FieldWrite<'a> {
    w: &'a mut Write,
    offset: u32,
    mask: u32,
}
impl<'a> FieldWrite<'a> {
    pub fn set_bit(self) -> &'a mut Write {
        unsafe { self.bits(1) }
    }
    pub fn clear_bit(self) -> &'a mut Write {
        unsafe { self.bits(0) }
    }
    pub unsafe fn bits(self, value: u8) -> &'a mut Write {
        self.w.0 = (self.w.0 & !(self.mask << self.offset))
            | ((value as u32 & self.mask) << self.offset);
        self.w
    }
}
macro_rules! field {
    ($name:ident,$offset:expr,$mask:expr) => {
        pub fn $name(&mut self) -> FieldWrite<'_> {
            FieldWrite {
                w: self,
                offset: $offset,
                mask: $mask,
            }
        }
    };
}
impl Write {
    pub unsafe fn bits(&mut self, value: u32) -> &mut Self {
        self.0 = value;
        self
    }
    field!(sclksel, 0, 3);
    field!(hicken, 0, 1);
    field!(hexten, 16, 1);
    field!(pllen, 24, 1);
    field!(pllrcs, 16, 1);
    field!(pllhextdiv, 17, 1);
    field!(pllmult3_0, 18, 15);
    field!(pllmult5_4, 29, 3);
    field!(pll_fref, 24, 7);
}
#[path = "../src/hal/clock.rs"]
mod clock;

fn check(ready_after: Option<u32>, external: bool) {
    let crm = Crm::new(ready_after);
    let flash = Flash {
        psr: Register::new(3, 0, None),
    };
    clock::configure_system_clock(&crm, &flash);
    let cfg = crm.cfg.value.get();
    assert_eq!((cfg >> 16) & 1, u32::from(external));
    assert_eq!((cfg >> 2) & 3, 2);
    assert_eq!(flash.psr.value.get(), 0x153);
    assert_eq!(
        (crm.pll.value.get() >> 24) & 7,
        if external { 2 } else { 0 }
    );
    let raw = ((cfg >> 18) & 15) | (((cfg >> 29) & 3) << 4);
    let mult = if raw >= 15 { raw + 1 } else { raw + 2 };
    assert_eq!(
        (if external { 8_000_000 } else { 4_000_000 }) * mult,
        clock::SCLK_HZ
    );
    assert!(crm.ctrl.polls.get() < 0x10010);
    if !external {
        assert_eq!(crm.ctrl.value.get() & (1 << 16), 0);
    }
}
#[test]
fn external_ready_immediately() {
    check(Some(0), true);
}
#[test]
fn external_ready_after_delay() {
    check(Some(50), true);
}
#[test]
fn external_absent_falls_back_without_hanging() {
    check(None, false);
}
#[test]
fn external_too_late_falls_back() {
    check(Some(0x10010), false);
}
