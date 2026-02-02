use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod riscv;

#[cfg(feature = "riscv-rt")]
mod riscv_rt;

/// Attribute-like macro that implements the traits of the `riscv-types` crate for a given enum.
///
/// As these traits are unsafe, the macro must be called with the `unsafe` keyword followed by the trait name.
/// In this way, we warn callers that they must comply with the requirements of the trait.
///
/// The trait name must be one of `ExceptionNumber`, `CoreInterruptNumber`, `ExternalInterruptNumber`,
/// `PriorityNumber`, or `HartIdNumber`.
///
/// # Note
///
/// Crates using this macro must depend on the `riscv` crate, as the generated code references it.
///
/// If the `rt` feature is enabled, the generated code may also include the necessary runtime support
/// for interrupt and exception handling. Thus, the calling crate must also depend on the `riscv-rt` crate.
///
/// # Safety
///
/// The struct to be implemented must comply with the requirements of the specified trait.
///
/// # Example
///
/// ```rust,ignore,no_run
/// use riscv::*;
///
/// #[repr(usize)]
/// #[pac_enum(unsafe ExceptionNumber)]
/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// enum Exception {
///     E1 = 1,
///     E3 = 3,
/// }
///
/// assert_eq!(Exception::E1.number(), 1);
/// assert_eq!(Exception::E3.number(), 3);
///
/// assert_eq!(Exception::from_number(1), Ok(Exception::E1));
/// assert_eq!(Exception::from_number(2), Err(2));
/// assert_eq!(Exception::from_number(3), Ok(Exception::E3));
///
/// assert_eq!(Exception::MAX_EXCEPTION_NUMBER, 3);
///```
#[proc_macro_attribute]
pub fn pac_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let pac_enum = riscv::PacEnumItem::new(&input);

    let attr = parse_macro_input!(attr as riscv::PacTrait);

    let trait_impl = pac_enum.impl_trait(&attr);
    quote! {
        #input
        #(#trait_impl)*
    }
    .into()
}

/// Attribute to mark which function will be called before jumping to the entry point.
/// You must enable the `post-init` feature in the `riscv-rt` crate to use this macro.
///
/// In contrast with `__pre_init`, this function is called after the static variables
/// are initialized, so it is safe to access them. It is also safe to run Rust code.
///
/// The function must have the signature of `[unsafe] fn([usize])`, where the argument
/// corresponds to the hart ID of the current hart. This is useful for multi-hart systems
/// to perform hart-specific initialization.
///
/// # IMPORTANT
///
/// This attribute can appear at most *once* in the dependency graph.
///
/// # Examples
///
/// ```
/// #[riscv_macros::post_init]
/// unsafe fn before_main(hart_id: usize) {
///     // do something here
/// }
/// ```
#[cfg(feature = "riscv-rt")]
#[proc_macro_attribute]
pub fn post_init(args: TokenStream, input: TokenStream) -> TokenStream {
    riscv_rt::Fn::post_init(args, input)
}

/// Temporary patch macro to deal with LLVM bug.
///
/// # Note
///
/// This macro is intended to be used internally by the `riscv-rt` crate. Do not use it directly in your code.
#[cfg(feature = "riscv-rt")]
#[proc_macro]
pub fn rvrt_llvm_arch_patch(_input: TokenStream) -> TokenStream {
    let q = if let Ok(arch) = std::env::var("RISCV_RT_LLVM_ARCH_PATCH") {
        let patch = format!(".attribute arch,\"{arch}\"");
        quote! { core::arch::global_asm!{#patch} }
    } else {
        quote!(compile_error!("RISCV_RT_LLVM_ARCH_PATCH is not set"))
    };
    q.into()
}

/// Generates assembly code required for the default handling of traps.
///
/// The main routine generated is `_default_start_trap`. If no `_start_trap` function
/// is defined, the linker will use this function as the default trap entry point.
///
/// If the `pre-default-start-trap` feature is enabled, the generated code will also
/// include a call to a user-defined function `_pre_default_start_trap` at the beginning
/// of the `_default_start_trap` routine.
///
/// If the `rt-v-trap` feature is enabled, the macro will also include the assembly code
/// for the `_start_DefaultInterrupt_trap` and `_continue_interrupt_trap` routines, which
/// are required for handling core interrupts in vectored trap mode.
///
/// # Note
///
/// This macro is intended to be used internally by the `riscv-rt` crate. Do not use it directly in your code.
#[cfg(feature = "riscv-rt")]
#[proc_macro]
pub fn rvrt_default_start_trap(_input: TokenStream) -> TokenStream {
    match riscv_rt::asm::RiscvArch::try_from_env() {
        Some(arch) => arch.default_start_trap().into(),
        None => quote! {
            compile_error!("RISCV_RT_BASE_ISA environment variable is not set or is invalid");
        }
        .into(),
    }
}
