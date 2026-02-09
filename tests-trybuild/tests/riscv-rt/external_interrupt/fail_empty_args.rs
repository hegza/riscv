#[riscv_rt::external_interrupt]
pub const async extern "Rust" fn my_interrupt<T>(_a: T, _b: ...) -> usize {}

fn main() {}
