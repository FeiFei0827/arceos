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
    println!("Load payload ...\n");
    println!("__________________________________________________________\n");
    for i in 0..app_info.apps_num {
        let app_size = app_info.app_size[i];
        let app_start = app_info.app_start[i];
        let code = unsafe {core::slice::from_raw_parts(app_start, app_size)};
        println!("load app{}, size:{}", i, app_size);
        println!("content: {:?}\n", code);
        println!("__________________________________________________________\n");
    }

    println!("Load payload ok!");

}

#[inline]
fn bytes_to_usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.try_into().unwrap())
}