extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use hermit_dtb::Dtb;

pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}

pub struct MiddleDtb<'a> {
    dtb: Option<Dtb<'a>>,
}
//not a good name
#[derive(Debug)]
pub enum MiddleDtbError {
    BAD,
}

impl<'ga> MiddleDtb<'ga> {
    pub fn new(dtb_pa: usize) -> Self {
        Self {
            dtb: unsafe { Dtb::from_raw(dtb_pa as *const u8) },
        }
    }

    pub fn parse_dtb(&self) -> Result<DtbInfo, MiddleDtbError> {

        let dtb =self.dtb.as_ref().unwrap()
        .enum_subnodes("/")//turn EnumSubnodesITer
        .filter(|&name| name.starts_with("memory"))//Creates an iterator which uses a closure to determine if an element should be yielded.
        .map(|name| self.parse_property(&name))
        .nth(0);
    //https://docs.rs/hermit-dtb/0.1.1/hermit_dtb/struct.Dtb.html

    
        match dtb {
            Some((memory_addr, memory_size)) => {
                let mmio_regions = self.parse_virtio_mmio();
                Ok(DtbInfo {
                    memory_addr,
                    memory_size,
                    mmio_regions,
                })
            }
            None => {
                Err(MiddleDtbError::BAD)
            }
        }
    }

    fn parse_virtio_mmio(&self) -> Vec<(usize, usize)> {
        self.dtb.as_ref().unwrap()
            .enum_subnodes("/src")
            .filter(|&name| name.starts_with("virtio_mmio"))
            .map(|name| self.parse_property(&["/src", name].join("/")))
            .collect()
    }

    fn parse_reg_bytes(&self, reg: &[u8]) -> (usize, usize) {
        (
            usize::from_be_bytes(reg[..8].to_owned().try_into().unwrap()),
            usize::from_be_bytes(reg[8..].to_owned().try_into().unwrap()),
        )
    }
    fn parse_property(&self, path: &str) -> (usize, usize) {
        match self.dtb.as_ref().unwrap().get_property(&path, "reg") {
            Some(reg) => self.parse_reg_bytes(reg),
            _ => (0, 0),
        }
    }


}