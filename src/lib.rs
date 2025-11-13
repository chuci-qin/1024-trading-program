//! 1024EX Trading Program
//! 
//! Phase 2: 开仓资金托管机制 - 确保 PostgreSQL总余额 = 链上USDC 永远相等
//! 
//! 核心特性：
//! - 开仓时锁定USDC到Program托管账户
//! - 平仓时返还USDC + 盈亏
//! - 强平机制
//! - Smart Hedge集成
//! - 多级扣款和传统ADL
//! 
//! GitHub: https://github.com/chuciqin/1024-trading-program
//! Version: 2.0.0
//! Author: Chuci Qin

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

// Trading Program ID (已部署到1024Chain Testnet)
// 部署日期: 2025-11-13
// Transaction: 3yhgKi33Vm5RUkXJoqggJ9ewp42j3ZsJhWodYyUfvckLvH2pg4SzwTruWaXc4PCDsDosTgpdsiy9pmq1mnePZuJS
solana_program::declare_id!("E3ea5jEUvTojcKiJWayNVTJ16gU52zkfLJArgudAUCFw");

// Program入口点
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process_instruction(program_id, accounts, instruction_data)
}

// 导出公共类型
pub use error::TradingError;
pub use instruction::TradingInstruction;
pub use state::{
    TradingVault, UserPosition, Side, MarginMode, LiquidationStatus,
};

