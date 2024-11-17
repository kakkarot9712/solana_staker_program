# Solana Staking Program
A Solana-based program that allows users to stake and unstake SPL tokens and Metaplex NFTs. This program is built using Solana's Anchor framework and leverages the SPL Token standard and Metaplex standards for NFTs.

## What is Staking
staking is a process where users lock up their cryptocurrency tokens in a blockchain network to support its operations, such as validating transactions and securing the network. In return, participants often earn rewards, typically in the form of additional tokens.

## Features
- SPL Token Staking: Stake SPL tokens to earn rewards or hold for specific purposes.
- NFT Staking: Stake Metaplex-standard NFTs securely.
- Unstaking: Seamlessly unstake SPL tokens and NFT

## Project Structure
- programs/: Contains the Anchor-based Solana program.
- tests/: Includes integration tests for the program.

## Prerequisites
- [Rust](https://www.rust-lang.org/tools/install)
- build-essantials pacakge if using ubuntu
```bash
sudo apt install build-essantials
```
- [Solana CLI](https://docs.solanalabs.com/cli/install)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)

## Build and Deployment Instructions
- Install all required tools. See Above.
- Clone this repo
```bash
git clone https://github.com/kakkarot9712/solana_staker_program
cd solana_staker_program
```
- Install all dependencies
```bash
anchor build
```
- Deploy the program to Solana Devnet or Mainnet
```bash
anchor deploy
```

## Testing Instruction
- Tests are written for testing all functionalities of this contract.
- You need to setup `solana-test-validator` to run tests on Localnet.
- To test in Localnet first start `solana-test-validator` and execute below command
```bash
sh run_test.sh
```
