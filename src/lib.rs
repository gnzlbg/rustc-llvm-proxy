#![deny(warnings)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

extern crate libc;
extern crate libloading as lib;
extern crate llvm_sys;

use lib::Library;

mod path;
use path::find_lib_path;

lazy_static! {
    static ref SHARED_LIB: Library = {
        let lib_path = match find_lib_path() {
            Ok(path) => path,

            Err(error) => {
                eprintln!("{}", error);
                panic!();
            }
        };

        match Library::new(lib_path) {
            Ok(path) => path,

            Err(error) => {
                eprintln!("Unable to open LLVM shared lib: {}", error);
                panic!();
            }
        }
    };
}

pub mod proxy {
    use super::SHARED_LIB;

    use llvm_sys::prelude::*;
    use llvm_sys::target::*;
    use llvm_sys::target_machine::*;
    use llvm_sys::transforms::pass_manager_builder::*;
    use llvm_sys::*;

    macro_rules! create_proxy {
        ($name:ident ; $ret_ty:ty ; $($arg:ident : $arg_ty:ty),*) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name($($arg: $arg_ty),*) -> $ret_ty {
                let entrypoint = {
                    *SHARED_LIB
                        .get::<unsafe extern "C" fn($($arg: $arg_ty),*) -> $ret_ty>(stringify!($name).as_bytes())
                        .unwrap()
                };

                entrypoint($($arg),*)
            }
        };
    }

    include!("llvm_gen.rs");
}