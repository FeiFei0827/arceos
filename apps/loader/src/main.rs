#![feature(asm_const)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#[cfg(feature = "axstd")]
use axstd::{println,process::exit};
#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;
use std::vec::Vec;
use core::ptr;
const PLASH_START: usize = 0x22000000;
const LEN: usize = 8;

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];
struct AppHeader {
    apps_num: usize,
    app_size: Vec<usize>,
    app_start: Vec<*const u8>,
}
fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

fn abi_terminate(exit_num: i32) {
    println!("[ABI:Terminate] Terminate!!!");
    exit(exit_num);

}


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
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, abi_terminate as usize);

    println!("Execute app ...");
    let arg0: u8 = b'A';


    let app_start = PLASH_START as *const u8;
    let app_info = AppHeader::read_from_apps(app_start);
    let num = app_info.apps_num;
    println!("Load payload ...\n");
    for i in 0..num {
        let app_size = app_info.app_size[i];
        let app_start = app_info.app_start[i];
        let code = unsafe {
            core::slice::from_raw_parts(app_start, app_size)
        };
        println!("load app {}, size is {}", i, app_size);
        println!("content: {:?}\n", code);
    }
    // app running aspace
    // SBI(0x8000_0000) -> APP <- Kernel(0x8020_0000)
    // 0xffff_ffc0_0000_0000
    const RUN_START:usize= 0xffff_ffc0_8010_0000;
    // execute app
        let arg0: u8 = b'A';
    unsafe { core::arch::asm!("
        li      t0, {abi_num}
        slli    t0, t0, 3
        la      t1, {abi_table}
        add     t1, t1, t0
        ld      t1, (t1)
        jalr    t1
        li      t2, {run_start}
        jalr    t2
        j       .",
        run_start = const RUN_START,
        abi_table = sym ABI_TABLE,
        //abi_num = const SYS_HELLO,
        abi_num = const SYS_TERMINATE,
         in("a0") arg0,
)}
}
#[inline]
fn bytes_to_usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.try_into().unwrap())
}