//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system plonk
//! ```

// use alloy_sol_types::{sol, SolType};

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    include_elf, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use std::path::PathBuf;
use zktls_att_verification::verification_data::VerifyingDataOpt;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZKTLS_ELF: &[u8] = include_elf!("zktls-program");

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct EVMArgs {
    #[arg(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
    #[arg(long, default_value = "16")]
    zktls_length: u32,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

// sol! {
//     /// The public values encoded as a struct that can be easily deserialized inside Solidity.
//     struct PublicZkTLSValuesStruct {
//         bytes zktls_verification_key;
//         bytes records;
//     }
// }

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1ZktlsProofFixture {
    // zktls_verification_key: String,
    // records: String,
    vkey: String,
    proof: String,
}

fn load(length: u32, stdin: &mut SP1Stdin) {
    match length {
        16 => {
            let verifying_key =
                std::fs::read_to_string("fixtures/zktls/verifying_k256.key").unwrap();

            stdin.write(&verifying_key);

            let verifying_data =
                std::fs::read_to_string("fixtures/zktls/data/bench16.json").unwrap();

            let verifying_data: VerifyingDataOpt = serde_json::from_str(&verifying_data).unwrap();

            stdin.write(&verifying_data);
        }
        256 => {
            let verifying_key =
                std::fs::read_to_string("fixtures/zktls/verifying_k256.key").unwrap();

            stdin.write(&verifying_key);

            let verifying_data =
                std::fs::read_to_string("fixtures/zktls/data/bench256.json").unwrap();

            let verifying_data: VerifyingDataOpt = serde_json::from_str(&verifying_data).unwrap();

            stdin.write(&verifying_data);
        }
        1024 => {
            let verifying_key =
                std::fs::read_to_string("fixtures/zktls/verifying_k256.key").unwrap();

            stdin.write(&verifying_key);

            let verifying_data =
                std::fs::read_to_string("fixtures/zktls/data/bench1024.json").unwrap();

            let verifying_data: VerifyingDataOpt = serde_json::from_str(&verifying_data).unwrap();

            stdin.write(&verifying_data);
        }
        2048 => {
            let verifying_key =
                std::fs::read_to_string("fixtures/zktls/verifying_k256.key").unwrap();

            stdin.write(&verifying_key);

            let verifying_data =
                std::fs::read_to_string("fixtures/zktls/data/bench2048.json").unwrap();

            let verifying_data: VerifyingDataOpt = serde_json::from_str(&verifying_data).unwrap();

            stdin.write(&verifying_data);
        }
        _ => {
            eprintln!("Unsupported length: {}", length);
            std::process::exit(1);
        }
    }
}
fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = EVMArgs::parse();

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the program.
    let (pk, vk) = client.setup(ZKTLS_ELF);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    load(args.zktls_length, &mut stdin);

    println!("zktls verification length: {}", args.zktls_length);
    println!("Proof System: {:?}", args.system);

    // Generate the proof based on the selected proof system.
    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, &stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, &stdin).groth16().run(),
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, &vk, args.system);
}

/// Create a fixture for the given proof.
fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
) {
    // Deserialize the public values.
    let _bytes = proof.public_values.as_slice();

    // let PublicZkTLSValuesStruct {
    //     zktls_verification_key,
    //     records,
    // } = PublicZkTLSValuesStruct::abi_decode(bytes).unwrap();

    let fixture = SP1ZktlsProofFixture {
        // zktls_verification_key: zktls_verification_key.to_string(),
        // records: records.to_string(),
        vkey: vk.bytes32().to_string(),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // println!("Zktls Verification Key: {}", fixture.zktls_verification_key);

    // The public values are the values which are publicly committed to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.

    // println!("Public Records: {}", fixture.records);

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
