pub extern crate paste;
pub extern crate rb_sys;

#[macro_export]
macro_rules! call_without_gvl {
    ($func:expr, args: ($($arg:expr, $ty:ty),+), return_type: $return_ty:ty) => {
        {
            lucchetto::paste::paste! {
                let mut result: $return_ty = Default::default();
                let data: ( $($ty,)+ *mut $return_ty ) = ($($arg,)+ &mut result);

                let data_ptr: *mut std::ffi::c_void = &data as *const _ as *mut _;
                std::mem::forget(data);

                unsafe extern "C" fn __decl_macro_anon_wrapper(data: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
                        let data = data as *mut ( $($ty,)+ *mut $return_ty );

                        let ( $([<__ $arg _name>],)+ result_output) = data.read();
                        let result = $func( $( [<__ $arg _name>], )+);
                        std::ptr::write_volatile(result_output, result);
                        std::ptr::null_mut()
                    }

                unsafe {
                    lucchetto::rb_sys::rb_thread_call_without_gvl(Some(__decl_macro_anon_wrapper), data_ptr, None, std::ptr::null_mut());
                }
                result
            }
        }
    };
}

pub use lucchetto_macros::without_gvl;