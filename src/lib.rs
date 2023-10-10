pub extern crate paste;
pub extern crate rb_sys;
pub extern crate static_assertions;

use std::{sync::Arc, rc::Rc};

pub use lucchetto_macros::without_gvl;

pub trait GvlSafe {}

macro_rules! impl_safe {
    ($($ty:ty),+) => {
        $(
            impl GvlSafe for $ty {}
        )+
    };
}

impl_safe!(String, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, &str);

macro_rules! impl_safe_1_ty {
    ($($ty:ty),+) => {
        $(
            impl<T: GvlSafe> GvlSafe for $ty {}
        )+
    };
}

impl<T: GvlSafe> GvlSafe for &[T] {}
impl_safe_1_ty!(Vec<T>, Arc<T>, Rc<T>, Box<T>, Option<T>);

#[macro_export]
macro_rules! call_without_gvl {
    ($func:expr, args: ($($arg:expr, $ty:ty),+), return_type: $return_ty:ty) => {
        {
            lucchetto::paste::paste! {
                let mut result: Option<$return_ty> = None;
                $(lucchetto::static_assertions::assert_impl_all!($ty: lucchetto::GvlSafe);)+
                lucchetto::static_assertions::assert_impl_all!($return_ty: lucchetto::GvlSafe);
                let data: ( $($ty,)+ &mut Option<$return_ty> ) = ($($arg,)+ &mut result);

                let data_ptr: *mut std::ffi::c_void = &data as *const _ as *mut _;
                std::mem::forget(data);

                unsafe extern "C" fn __decl_macro_anon_wrapper(data: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
                        let data = data as *mut ( $($ty,)+ &mut Option<$return_ty> );

                        let ( $([<__ $arg _name>],)+ result_output) = data.read();
                        let result = $func( $( [<__ $arg _name>], )+);
                        *result_output = Some(result);
                        std::ptr::null_mut()
                    }

                unsafe {
                    lucchetto::rb_sys::rb_thread_call_without_gvl(Some(__decl_macro_anon_wrapper), data_ptr, None, std::ptr::null_mut());
                }
                result.unwrap()
            }
        }
    };
}
