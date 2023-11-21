#![feature(asm_const)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::vec::Vec;
use core::ptr;
const PLASH_START: usize = 0x22000000;
const LEN: usize = 8;
struct AppHeader {
    apps_num: usize,
    app_size: Vec<usize>,
    app_start: Vec<*const u8>,
}
//Vec

impl AppHeader {
    fn new(num:usize,size:Vec<usize>,start: Vec<*const u8>)->Self{
        AppHeader {
            apps_num: (num),
            app_size: (size),
            app_start: (start),
            }
    }
    fn read_from_apps (apps_start: *const u8) -> Self {
        let mut offset = 0;
        let usize_val= unsafe{ core::slice::from_raw_parts(apps_start.offset(offset as isize), LEN) };
        let apps_num = bytes_to_usize(usize_val);

        let mut app_size = Vec::new();
        let mut app_start = Vec::new();

        for _ in 0..apps_num {
            offset += LEN;
            let val = bytes_to_usize(unsafe{ core::slice::from_raw_parts(apps_start.offset(offset as isize), LEN) });
            app_size.push(val);
        }

        let mut app_start_offset = offset + LEN;

        for i in 0..apps_num {
            
            app_start.push((PLASH_START + app_start_offset )as *const u8);
            app_start_offset += app_size[i];
        }

        AppHeader { 
            apps_num,
            app_size, 
            app_start,
        }
    }

}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let apps_start = PLASH_START as *const u8;
    let app_info = AppHeader::read_from_apps(apps_start);
    println!("Load payload ok!");
    // app running aspace
    // SBI(0x8000_0000) -> APP <- Kernel(0x8020_0000)
    // 0xffff_ffc0_0000_0000
    const RUN_START:usize= 0xffff_ffc0_8010_0000;

    for i in 0..app_info.apps_num {
        let app_size = app_info.app_size[i];
        let app_start = app_info.app_start[i];
        let load = unsafe {
            core::slice::from_raw_parts(app_start, app_size)
        };
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, app_size)
        };
        run_code.copy_from_slice(load);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
        println!("Execute app ...\n");

        // execute app
        unsafe { core::arch::asm!("
            li      t2, {run_start}
            jalr    t2",
            run_start = const RUN_START,
        )}
        let clear_value = 0;
        unsafe {
            ptr::write_bytes(run_code.as_mut_ptr(), clear_value, run_code.len());
        }
    }

}

#[inline]
fn bytes_to_usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.try_into().unwrap())
}