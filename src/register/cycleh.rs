//! cycleh register
//! Shadow of mcycleh register (rv32)
//! must have `scounter::cy` or `mcounteren::cy` bit enabled depending on whether
//! S-mode is implemented or not

read_csr_as_usize_rv32!(0xC80, __read_cycleh);
