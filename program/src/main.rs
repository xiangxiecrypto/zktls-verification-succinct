//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
use zktls_att_verification::verification_data::VerifyingDataOpt;
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let verifying_key: String = sp1_zkvm::io::read();
    let verifying_data: VerifyingDataOpt = sp1_zkvm::io::read();

    let _ = verifying_data.verify(&verifying_key).is_ok();

    sp1_zkvm::io::commit(&verifying_key);
    sp1_zkvm::io::commit(&verifying_data.get_records());
}
