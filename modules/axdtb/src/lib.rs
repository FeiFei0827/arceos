#![no_std]

mod dtb;
pub use dtb::{MiddleDtb, DtbInfo, MiddleDtbError};
pub fn parse_dtb(dtb_pa: usize) -> Result<DtbInfo, MiddleDtbError> {
    MiddleDtb::new(dtb_pa).parse_dtb()
}