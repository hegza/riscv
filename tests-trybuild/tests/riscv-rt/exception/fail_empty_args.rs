#[riscv_rt::exception]
pub const async extern "Rust" fn my_exception<T>(_a: T, _b: usize, _c: ...) -> usize {}

fn main() {}
