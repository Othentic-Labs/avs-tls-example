// This example demonstrates how to verify a presentation. See `present.rs` for
// an example of how to build a presentation from an attestation and connection
// secrets.

use std::time::Duration;

use tlsn_core::{
    presentation::{Presentation, PresentationOutput},
    signing::VerifyingKey,
};

use clap::Parser;
use tlsn_examples::ExampleType;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// What data to notarize
    #[clap(default_value_t, value_enum)]
    example_type: ExampleType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    verify_presentation(&args.example_type).await
}

async fn verify_presentation(example_type: &ExampleType) -> Result<(), Box<dyn std::error::Error>> {
    // Read the presentation from disk.
    let presentation_path = tlsn_examples::get_file_path(example_type, "presentation");

    let presentation: Presentation = bincode::deserialize(&std::fs::read(presentation_path)?)?;

    let provider = tlsn_examples::get_crypto_provider_with_server_fixture();

    let VerifyingKey {
        alg,
        data: key_data,
    } = presentation.verifying_key();

    println!(
        "Verifying presentation with {alg} key: {}\n\n**Ask yourself, do you trust this key?**\n",
        hex::encode(key_data)
    );

    // Verify the presentation.
    let PresentationOutput {
        server_name,
        connection_info,
        transcript,
        ..
    } = presentation.verify(&provider).unwrap();

    // The time at which the connection was started.
    let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(connection_info.time);
    let server_name = server_name.unwrap();
    let mut partial_transcript = transcript.unwrap();
    // Set the unauthenticated bytes so they are distinguishable.
    partial_transcript.set_unauthed(b'X');

    let sent = String::from_utf8_lossy(partial_transcript.sent_unsafe());
    let recv = String::from_utf8_lossy(partial_transcript.received_unsafe());

    println!("-------------------------------------------------------------------");
    println!(
        "Successfully verified that the data below came from a session with {server_name} at {time}.",
    );
    println!("Note that the data which the Prover chose not to disclose are shown as X.\n");
    println!("Data sent:\n");
    println!("{}\n", sent);
    println!("Data received:\n");
    println!("{}\n", recv);
    println!("-------------------------------------------------------------------");

    Ok(())
}
