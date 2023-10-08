use lucchetto::without_gvl;
use magnus::{define_global_function, function};

#[without_gvl]
fn slow_func_no_lock(input: String) -> String {
    std::thread::sleep(std::time::Duration::from_secs(2));
    input.len().to_string()
}

fn slow_func(input: String) -> String {
    std::thread::sleep(std::time::Duration::from_secs(2));
    input.len().to_string()
}

#[magnus::init]
fn init() {
    define_global_function("slow_func_no_lock", function!(slow_func_no_lock, 1));
    define_global_function("slow_func", function!(slow_func, 1));
}
