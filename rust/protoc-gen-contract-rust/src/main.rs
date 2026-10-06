//! `protoc-gen-contract-rust`: reads a `CodeGeneratorRequest` from stdin
//! and writes the `CodeGeneratorResponse` to stdout. See the library crate
//! for what it generates and the parameters it takes.

use std::io::{self, Read, Write};
use std::process::ExitCode;

use buffa::Message;

fn main() -> ExitCode {
    let mut input = Vec::new();
    if let Err(error) = io::stdin().read_to_end(&mut input) {
        eprintln!("protoc-gen-contract-rust: failed to read the request from stdin: {error}");
        return ExitCode::FAILURE;
    }
    let response = protoc_gen_contract_rust::run(&input);
    if let Err(error) = io::stdout().write_all(&response.encode_to_vec()) {
        eprintln!("protoc-gen-contract-rust: failed to write the response to stdout: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
