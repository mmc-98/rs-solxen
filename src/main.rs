use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::solana_sdk::{ system_program};
use anchor_client::{Client, Cluster};
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::compute_budget::ComputeBudgetInstruction;
use anchor_client::solana_sdk::signer::EncodableKey;
use anchor_lang::declare_program;
use crate::sol_xen_miner_0::client;
use dotenv::dotenv;
declare_program!(sol_xen_miner_0);
use ethaddr::Address;
use spl_memo::build_memo;
use std::fs;
use std::str;

 fn main() -> anyhow::Result<()> {

    dotenv().ok();

    let keypair = std::env::var("WALLET").expect("keypair must be set");
    let payer = Keypair::read_from_file(keypair).unwrap();

     let url = std::env::var("URL").expect("url must be set");
    let url = Cluster::Custom(
        url,
        "https://api.mainnet-beta.solana.com".to_string(),);

    let anchor_client = Client::new_with_options(
        url,
        &payer,
        CommitmentConfig::confirmed(),
    );

     let address_str = std::env::var("TOADDR").expect("address must be set");
     let address =   Address::from_str_checksum(&address_str).unwrap();

     let kind: u8 = 0;
     let program_id = sol_xen_miner_0::ID;

     let (global_xn_record_pda, _global_bump) = Pubkey::find_program_address(
         &[b"xn-miner-global", kind.to_be_bytes().as_slice()],
         &program_id
     );
     let (user_eth_xn_record_pda, _user_eth_bump) = Pubkey::find_program_address(
         &[
             b"xn-by-eth",
             &address.as_slice(),
             kind.to_be_bytes().as_slice(),
             program_id.as_ref()
         ],
         &program_id
     );

     let (user_sol_xn_record_pda, _user_sol_bump) = Pubkey::find_program_address(
         &[
             b"xn-by-sol",
             &payer.pubkey().to_bytes(),
             kind.to_be_bytes().as_slice(),
             program_id.as_ref()
         ],
         &program_id
     );

     let compute_budget_instruction_limit = ComputeBudgetInstruction::set_compute_unit_limit(1150000);
     let fee = std::env::var("FEE").unwrap().parse::<u64>().unwrap();
     let compute_budget_instruction_price = ComputeBudgetInstruction::set_compute_unit_price(fee);


     let program = anchor_client.program(sol_xen_miner_0::ID)?;

     let byte_content = fs::read("./message.txt").expect("Failed to read message.txt");
     let string_content = str::from_utf8(&byte_content);
     if string_content.unwrap().len() > 566 {
         panic!("message too long");
     }
     let signature = program
         .request()
         .instruction(compute_budget_instruction_limit)
         .instruction(compute_budget_instruction_price)
         .instruction(build_memo(string_content.unwrap().as_bytes(), &[]))
         .signer(&payer)
         .accounts(client::accounts::MineHashes {
             global_xn_record: global_xn_record_pda,
             xn_by_eth: user_eth_xn_record_pda,
             xn_by_sol: user_sol_xn_record_pda,
             user: payer.pubkey(),
             system_program: system_program::ID,
         })
         .args(client::args::MineHashes {
             eth_account: sol_xen_miner_0::types::EthAccount {
                 address: address.0,
                 address_str: address.to_string(),
             },
             _kind: kind,
         })
         // .simulate()
         .send()
         .unwrap();
     println!("{}", signature.to_string());
    Ok(())
}
