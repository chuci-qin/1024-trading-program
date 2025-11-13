//! 初始化Trading Vault示例

use anyhow::Result;
use borsh::BorshSerialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;
use trading_program::instruction::TradingInstruction;

fn main() -> Result<()> {
    println!("🏗️  1024 Trading Program - Initialize Vault");
    println!("==========================================");
    println!();
    
    // 配置
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://testnet-rpc.1024chain.com/rpc/".to_string());
    
    let program_id = std::env::var("PROGRAM_ID")
        .map(|s| Pubkey::from_str(&s).expect("Invalid PROGRAM_ID"))
        .unwrap_or_else(|_| {
            // 读取program-id.txt
            std::fs::read_to_string("program-id.txt")
                .ok()
                .and_then(|s| Pubkey::from_str(s.trim()).ok())
                .expect("No PROGRAM_ID env var or program-id.txt found")
        });
    
    let authority_path = std::env::var("AUTHORITY_KEYPAIR")
        .unwrap_or_else(|_| "../../1024-chain/settlement-authority.json".to_string());
    
    println!("Configuration:");
    println!("  RPC URL: {}", rpc_url);
    println!("  Program ID: {}", program_id);
    println!("  Authority: {}", authority_path);
    println!();
    
    // 读取authority keypair
    let authority = read_keypair_file(&authority_path)
        .map_err(|e| anyhow::anyhow!("Failed to read authority keypair: {}", e))?;
    
    println!("Authority Pubkey: {}", authority.pubkey());
    
    // 创建RPC client
    let rpc_client = RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    );
    
    // 检查余额
    let balance = rpc_client.get_balance(&authority.pubkey())?;
    println!("Authority Balance: {} SOL", balance as f64 / 1_000_000_000.0);
    
    if balance < 100_000_000 {
        return Err(anyhow::anyhow!("Insufficient balance (need at least 0.1 SOL)"));
    }
    
    println!();
    
    // 派生PDAs
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(
        &[b"trading_vault"],
        &program_id,
    );
    
    println!("Derived Accounts:");
    println!("  Trading Vault PDA: {}", vault_pda);
    println!();
    
    // TODO: 创建Vault USDC Token Account
    // 这里需要先创建Associated Token Account
    println!("⚠️  Warning: You need to manually create Vault USDC Account");
    println!("    Use: spl-token create-account <USDC_MINT>");
    println!();
    
    // 构造InitializeVault instruction
    let instruction_data = TradingInstruction::InitializeVault;
    let data = instruction_data.try_to_vec()?;
    
    // Note: 这里需要实际的vault_usdc_account地址
    let vault_usdc_account = Pubkey::new_unique(); // 临时placeholder
    
    let accounts = vec![
        AccountMeta::new(vault_pda, false),
        AccountMeta::new(vault_usdc_account, false),
        AccountMeta::new(authority.pubkey(), true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
    ];
    
    let instruction = Instruction {
        program_id,
        accounts,
        data,
    };
    
    // 创建并发送交易
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&authority.pubkey()),
        &[&authority],
        recent_blockhash,
    );
    
    println!("📤 Sending InitializeVault transaction...");
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    
    println!();
    println!("✅ Trading Vault initialized successfully!");
    println!();
    println!("Transaction Signature: {}", signature);
    println!("Trading Vault PDA: {}", vault_pda);
    println!();
    println!("View on Explorer:");
    println!("  https://testnet-scan.1024chain.com/tx/{}", signature);
    println!();
    
    Ok(())
}

