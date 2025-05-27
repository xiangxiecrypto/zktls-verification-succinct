//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

// use alloy_sol_types::SolType;
use clap::Parser;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use zktls_att_verification::verification_data::VerifyingDataOpt;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ZKTLS_ELF: &[u8] = include_elf!("zktls-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long, default_value = "16")]
    zktls_length: u32,
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
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    load(args.zktls_length, &mut stdin);
    // stdin.write(&args.n);

    println!("zktls verification length: {}", args.zktls_length);

    if args.execute {
        // Execute the program
        let (_, report) = client.execute(ZKTLS_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(ZKTLS_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
