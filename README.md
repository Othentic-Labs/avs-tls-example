# 🔐 TLS AVS Example
This project demonstrates how to build and verify MPC TLS sessions in an AVS using the Othentic Stack.


## Table of Contents

1. [Overview](#overview)
2. [Architecture](#usage)
3. [Prerequisites](#prerequisites)
4. [Installation](#installation)
5. [Usage](#usage)
6. [Usage with Docker](#usage)

## Overview
This project demonstrates a complete implementation of a TLS-MPC AVS. 

**Key Components:**

- **Notary Server** – Verifies TLS sessions and issues notarized transcripts.
- **Fixture Server** – Mock HTTPS endpoint for TLS interactions.
- **Prover (Execution Service)** –  Establishes a TLS session, generates `.tlsn` proof, uploads it to IPFS.
- **Verifier (Validation Service)** – Downloads and verifies the proof before approving the task.


## Architecture

![image](https://github.com/user-attachments/assets/9c490a03-9751-4668-b9ed-c5ff109577a1)

- The Performer node executes tasks using the Task Execution Service and sends the results to the p2p network.
- Attester Nodes validate task execution through the Validation Service. Based on the Validation Service's response, attesters sign the tasks. 

**In this AVS:**

1. Prover generates a notarized TLS transcript (.presentation.tlsn).

2. Transcript is uploaded to IPFS via Pinata.

3. Verifier downloads the file and validates the proof.

4. Based on the result, the AVS either approves or rejects the task.


## Prerequisites

- Rust (v 1.23 )
- Foundry
- [Docker](https://docs.docker.com/engine/install/)

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/Othentic-Labs/avs-tls-example.git
   cd avs-tls-example
   ```

2. Install Othentic CLI:

   ```bash
   npm i -g @othentic/othentic-cli
   ```

## Usage

### 1. Start the Notary Server
```bash
cd Execution_Service/tlsn-src/crates/notary/server
cargo run -r -- --tls-enabled false
```
The server will start listening on `0.0.0.0:7047`.

### 2. Start the Fixture Server
```bash
cd Execution_Service/tlsn-src/crates/server-fixture/server
PORT=4000 cargo run --release
```
Simulates an HTTPS server used by the prover to establish TLS sessions.

### 3. Run the Execution Service (Prover)
```bash
cd Execution_Service/
cargo build
# Use this command to run the prover directly
# RUST_LOG=debug SERVER_PORT=4000 cargo run --bin attestation-prove 
cargo run --bin Execution_Service
```
The Execution service will start on port 4003.

### 4. Trigger task execution with following command

```bash
curl -X POST http://localhost:4003/task/execute -H "Content-Type: application/json" -d "{}"
```

It will:
- Connect to the notary server
- Establish a TLS connection
- Create a notarized TLS transcript file (example-json.presentation.tlsn)
- Upload the file to the IPFS and Return the IPFS hash as Proof of Task

### 5. Run the Validation Service (Verifier)
```bash
cd Validation_Service/
cargo build
# Use this command to verify directly
# cargo run --bin attestation-verify
cargo run --bin Validation_Service
```

### 6. Validate the Proof of Task
Replace <proofOfTask> with the actual hash returned from the Execution Service:

```bash
curl -X POST http://localhost:4002/task/validate -H "Content-Type: application/json" -d '{"proofOfTask":"QmSURnBJXpcCKgahoFX559DbRreib2z9hpq4MYzwrX4v2g"}'
```


## Usage with Docker

Follow the steps in the official documentation's [Quickstart](https://docs.othentic.xyz/main/avs-framework/quick-start#steps) Guide for setup and deployment.

If you already have all the information required to run the AVS, simply copy the .env file into your project directory and then run:
```bash
docker-compose up --build
```

### Next
Modify the different configurations, tailor the task execution logic as per your use case, and run the AVS.

Happy Building! 🚀

