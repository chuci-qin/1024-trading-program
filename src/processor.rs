//! Trading Program Instruction Processing

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    error::TradingError,
    instruction::TradingInstruction,
    state::{TradingVault, UserPosition, Side},
    utils::*,
};

/// 授权的Relayer公钥（管理员）
pub const AUTHORIZED_RELAYER: &str = "J1Szw8HZYL95NvYUsNhg3e6NzKQLUZ9UxQsKg4hsQnad";

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TradingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
    match instruction {
        TradingInstruction::InitializeVault => {
            msg!("Instruction: InitializeVault");
            process_initialize_vault(program_id, accounts)
        }
        TradingInstruction::LockMargin {
            account_id,
            market,
            side,
            size_e6,
            entry_price_e6,
            leverage,
            margin_mode,
        } => {
            msg!("Instruction: LockMargin");
            process_lock_margin(
                program_id,
                accounts,
                account_id,
                market,
                side,
                size_e6,
                entry_price_e6,
                leverage,
                margin_mode,
            )
        }
        TradingInstruction::UnlockMargin {
            account_id,
            market,
            close_size_e6,
            exit_price_e6,
        } => {
            msg!("Instruction: UnlockMargin");
            process_unlock_margin(
                program_id,
                accounts,
                account_id,
                market,
                close_size_e6,
                exit_price_e6,
            )
        }
        TradingInstruction::Liquidate {
            account_id,
            market,
            liquidation_price_e6,
        } => {
            msg!("Instruction: Liquidate");
            process_liquidate(
                program_id,
                accounts,
                account_id,
                market,
                liquidation_price_e6,
            )
        }
        TradingInstruction::PartialCloseForHedge {
            account_id,
            market,
            close_ratio,
            exit_price_e6,
            hedge_mode,
        } => {
            msg!("Instruction: PartialCloseForHedge");
            process_partial_close_for_hedge(
                program_id,
                accounts,
                account_id,
                market,
                close_ratio,
                exit_price_e6,
                hedge_mode,
            )
        }
        TradingInstruction::CreateReentryPosition {
            account_id,
            market,
            entry_price_e6,
            pool_created_at,
        } => {
            msg!("Instruction: CreateReentryPosition");
            process_create_reentry_position(
                program_id,
                accounts,
                account_id,
                market,
                entry_price_e6,
                pool_created_at,
            )
        }
        TradingInstruction::ExecuteTpSl {
            account_id,
            market,
            exit_price_e6,
            is_take_profit,
            pool_created_at,
        } => {
            msg!("Instruction: ExecuteTpSl");
            process_execute_tpsl(
                program_id,
                accounts,
                account_id,
                market,
                exit_price_e6,
                is_take_profit,
                pool_created_at,
            )
        }
        TradingInstruction::UpdatePosition {
            account_id,
            market,
            wallet,
            mark_price_e6,
        } => {
            msg!("Instruction: UpdatePosition");
            process_update_position(
                program_id,
                accounts,
                account_id,
                market,
                wallet,
                mark_price_e6,
            )
        }
        TradingInstruction::WithdrawInsuranceFund { amount_e6 } => {
            msg!("Instruction: WithdrawInsuranceFund");
            process_withdraw_insurance_fund(program_id, accounts, amount_e6)
        }
    }
}

/// 初始化Trading Vault（仅一次）
fn process_initialize_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let trading_vault_account = next_account_info(account_iter)?;
    let vault_usdc_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;
    let _token_program = next_account_info(account_iter)?;
    let _rent = next_account_info(account_iter)?;
    
    msg!("Initializing Trading Vault...");
    
    // 验证authority签名
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // 派生Trading Vault PDA
    let (expected_vault_pda, bump) = Pubkey::find_program_address(
        &[b"trading_vault"],
        program_id,
    );
    
    if trading_vault_account.key != &expected_vault_pda {
        msg!("Error: Vault PDA mismatch");
        return Err(TradingError::InvalidVaultAccount.into());
    }
    
    // 验证account尚不存在
    if trading_vault_account.lamports() > 0 {
        return Err(TradingError::AccountAlreadyExists.into());
    }
    
    // 创建Trading Vault account
    let rent = Rent::get()?;
    let space = TradingVault::SIZE;
    let required_lamports = rent.minimum_balance(space);
    
    msg!("Creating Trading Vault PDA...");
    msg!("  Space: {} bytes", space);
    msg!("  Rent: {} lamports", required_lamports);
    
    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            trading_vault_account.key,
            required_lamports,
            space as u64,
            program_id,
        ),
        &[
            authority.clone(),
            trading_vault_account.clone(),
            system_program.clone(),
        ],
        &[&[b"trading_vault", &[bump]]],
    )?;
    
    // 初始化Vault数据
    let vault = TradingVault::new(*authority.key, bump);
    let serialized = vault.try_to_vec()
        .map_err(|_| TradingError::SerializationError)?;
    
    trading_vault_account.data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    
    msg!("✅ Trading Vault initialized!");
    msg!("  PDA: {}", trading_vault_account.key);
    msg!("  Vault USDC Account: {}", vault_usdc_account.key);
    
    Ok(())
}

/// 开仓锁定保证金
#[allow(clippy::too_many_arguments)]
fn process_lock_margin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    account_id: String,
    market: String,
    side: Side,
    size_e6: i64,
    entry_price_e6: i64,
    leverage: u32,
    margin_mode: crate::state::MarginMode,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let user_position_account = next_account_info(account_iter)?;
    let user = next_account_info(account_iter)?;
    let user_usdc_account = next_account_info(account_iter)?;
    let vault_usdc_account = next_account_info(account_iter)?;
    let trading_vault_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;
    
    msg!("Lock Margin - Opening Position");
    msg!("  Account: {}", account_id);
    msg!("  Market: {}", market);
    msg!("  Side: {:?}", side);
    msg!("  Size: {}", size_e6);
    msg!("  Price: {}", entry_price_e6);
    msg!("  Leverage: {}x", leverage);
    
    // 验证输入
    validate_account_id(&account_id)?;
    validate_market(&market)?;
    validate_size(size_e6)?;
    validate_price(entry_price_e6)?;
    validate_leverage(leverage)?;
    
    // 验证user签名
    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // 计算所需保证金
    let im_e6 = calculate_initial_margin(size_e6, entry_price_e6, leverage)?;
    let mm_e6 = calculate_maintenance_margin(im_e6);
    
    msg!("  IM: {} USDC", im_e6 as f64 / 1_000_000.0);
    msg!("  MM: {} USDC", mm_e6 as f64 / 1_000_000.0);
    
    // 派生UserPosition PDA
    let (expected_position_pda, position_bump) = Pubkey::find_program_address(
        &[
            b"position",
            user.key.as_ref(),
            account_id.as_bytes(),
            market.as_bytes(),
        ],
        program_id,
    );
    
    if user_position_account.key != &expected_position_pda {
        msg!("Error: Position PDA mismatch");
        return Err(TradingError::InvalidPositionAccount.into());
    }
    
    // 检查持仓是否存在
    let is_new_position = user_position_account.lamports() == 0;
    
    if is_new_position {
        // 创建新持仓
        msg!("Creating new position...");
        
        let rent = Rent::get()?;
        let space = UserPosition::MAX_SIZE;
        let required_lamports = rent.minimum_balance(space);
        
        invoke_signed(
            &system_instruction::create_account(
                user.key,
                user_position_account.key,
                required_lamports,
                space as u64,
                program_id,
            ),
            &[
                user.clone(),
                user_position_account.clone(),
                system_program.clone(),
            ],
            &[&[
                b"position",
                user.key.as_ref(),
                account_id.as_bytes(),
                market.as_bytes(),
                &[position_bump],
            ]],
        )?;
        
        // 初始化Position数据
        let position = UserPosition::new(
            *user.key,
            account_id.clone(),
            market.clone(),
            side,
            size_e6,
            entry_price_e6,
            leverage,
            margin_mode,
            im_e6,
            mm_e6,
            position_bump,
        );
        
        let serialized = position.try_to_vec()
            .map_err(|_| TradingError::SerializationError)?;
        
        user_position_account.data.borrow_mut()[..serialized.len()]
            .copy_from_slice(&serialized);
    } else {
        // 更新现有持仓（加仓）
        msg!("Adding to existing position...");
        
        let mut position = UserPosition::try_from_slice(&user_position_account.data.borrow())
            .map_err(|_| TradingError::DeserializationError)?;
        
        // 验证方向一致
        if position.side != side {
            msg!("Error: Cannot add to position with different side");
            return Err(TradingError::InvalidSize.into());
        }
        
        // 更新持仓
        position.size_e6 = safe_add_i64(position.size_e6, size_e6)?;
        position.locked_usdc_e6 = safe_add_i64(position.locked_usdc_e6, im_e6)?;
        position.mm_e6 = safe_add_i64(position.mm_e6, mm_e6)?;
        
        // 重新计算均价
        let total_notional = (position.entry_price_e6 as i128 * (position.size_e6 - size_e6) as i128
            + entry_price_e6 as i128 * size_e6 as i128) / 1_000_000;
        position.entry_price_e6 = (total_notional * 1_000_000 / position.size_e6 as i128) as i64;
        
        position.updated_at = solana_program::clock::Clock::get()?.unix_timestamp;
        
        let serialized = position.try_to_vec()
            .map_err(|_| TradingError::SerializationError)?;
        
        user_position_account.data.borrow_mut()[..serialized.len()]
            .copy_from_slice(&serialized);
    }
    
    // SPL Token Transfer: 用户 → Vault
    msg!("Transferring {} USDC to vault...", im_e6 as f64 / 1_000_000.0);
    
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            user_usdc_account.key,
            vault_usdc_account.key,
            user.key,
            &[],
            im_e6 as u64,
        )?,
        &[
            user_usdc_account.clone(),
            vault_usdc_account.clone(),
            user.clone(),
            token_program.clone(),
        ],
        &[],
    )?;
    
    // 更新Trading Vault全局状态
    let mut vault = TradingVault::try_from_slice(&trading_vault_account.data.borrow())
        .map_err(|_| TradingError::DeserializationError)?;
    
    vault.total_locked_usdc_e6 = safe_add_i64(vault.total_locked_usdc_e6, im_e6)?;
    
    if is_new_position {
        vault.total_positions += 1;
    }
    
    vault.updated_at = solana_program::clock::Clock::get()?.unix_timestamp;
    
    let serialized = vault.try_to_vec()
        .map_err(|_| TradingError::SerializationError)?;
    
    trading_vault_account.data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    
    // Emit事件
    msg!("POSITION_OPENED|account:{}|market:{}|side:{:?}|size:{}|price:{}|leverage:{}|im:{}|mm:{}",
        account_id, market, side, size_e6, entry_price_e6, leverage, im_e6, mm_e6);
    
    msg!("✅ Position opened successfully!");
    msg!("  Locked USDC: {} (e6)", im_e6);
    msg!("  Total Vault USDC: {} (e6)", vault.total_locked_usdc_e6);
    
    Ok(())
}

/// 平仓返还保证金
fn process_unlock_margin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    account_id: String,
    market: String,
    close_size_e6: i64,
    exit_price_e6: i64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let user_position_account = next_account_info(account_iter)?;
    let user = next_account_info(account_iter)?;
    let user_usdc_account = next_account_info(account_iter)?;
    let vault_usdc_account = next_account_info(account_iter)?;
    let trading_vault_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    
    msg!("Unlock Margin - Closing Position");
    msg!("  Account: {}", account_id);
    msg!("  Market: {}", market);
    msg!("  Close Size: {}", close_size_e6);
    msg!("  Exit Price: {}", exit_price_e6);
    
    // 验证输入
    validate_size(close_size_e6)?;
    validate_price(exit_price_e6)?;
    
    // 验证user签名
    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // 读取持仓
    let mut position = UserPosition::try_from_slice(&user_position_account.data.borrow())
        .map_err(|_| TradingError::DeserializationError)?;
    
    // 验证持仓归属
    if position.wallet != *user.key {
        return Err(TradingError::InvalidPositionAccount.into());
    }
    
    // 验证平仓数量
    if close_size_e6 > position.size_e6.abs() {
        return Err(TradingError::InvalidSize.into());
    }
    
    // 计算Realized PnL
    let is_long = matches!(position.side, Side::Buy);
    let realized_pnl = calculate_realized_pnl(
        is_long,
        position.entry_price_e6,
        exit_price_e6,
        close_size_e6,
    )?;
    
    msg!("  Realized PnL: {} USDC", realized_pnl as f64 / 1_000_000.0);
    
    // 计算释放的保证金
    let close_ratio = close_size_e6 as i128 * 1_000_000 / position.size_e6.abs() as i128;
    let released_im = (position.locked_usdc_e6 as i128 * close_ratio / 1_000_000) as i64;
    
    // 计算返还金额
    let return_amount = safe_add_i64(released_im, realized_pnl)?;
    
    if return_amount < 0 {
        msg!("Warning: Return amount is negative (loss), returning 0");
    }
    
    let actual_return = return_amount.max(0);
    
    msg!("  Released IM: {} USDC", released_im as f64 / 1_000_000.0);
    msg!("  Return Amount: {} USDC", actual_return as f64 / 1_000_000.0);
    
    // SPL Token Transfer: Vault → 用户
    if actual_return > 0 {
        // 派生Vault PDA以签名
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"trading_vault"],
            program_id,
        );
        
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                vault_usdc_account.key,
                user_usdc_account.key,
                &vault_pda,
                &[],
                actual_return as u64,
            )?,
            &[
                vault_usdc_account.clone(),
                user_usdc_account.clone(),
                trading_vault_account.clone(),
                token_program.clone(),
            ],
            &[&[b"trading_vault", &[vault_bump]]],
        )?;
    }
    
    // 更新持仓
    position.size_e6 = safe_sub_i64(position.size_e6, close_size_e6)?;
    position.locked_usdc_e6 = safe_sub_i64(position.locked_usdc_e6, released_im)?;
    position.mm_e6 = calculate_maintenance_margin(position.locked_usdc_e6);
    position.realized_pnl_e6 = safe_add_i64(position.realized_pnl_e6, realized_pnl)?;
    position.updated_at = solana_program::clock::Clock::get()?.unix_timestamp;
    
    let is_fully_closed = position.size_e6 == 0;
    
    if is_fully_closed {
        msg!("Position fully closed, clearing account...");
        // TODO: 关闭PDA账户并返还租金
    } else {
        let serialized = position.try_to_vec()
            .map_err(|_| TradingError::SerializationError)?;
        
        user_position_account.data.borrow_mut()[..serialized.len()]
            .copy_from_slice(&serialized);
    }
    
    // 更新Trading Vault
    let mut vault = TradingVault::try_from_slice(&trading_vault_account.data.borrow())
        .map_err(|_| TradingError::DeserializationError)?;
    
    vault.total_locked_usdc_e6 = safe_sub_i64(vault.total_locked_usdc_e6, released_im)?;
    
    if is_fully_closed {
        vault.total_positions = vault.total_positions.saturating_sub(1);
    }
    
    vault.cumulative_pnl_e6 = safe_add_i64(vault.cumulative_pnl_e6, realized_pnl)?;
    vault.updated_at = solana_program::clock::Clock::get()?.unix_timestamp;
    
    let serialized = vault.try_to_vec()
        .map_err(|_| TradingError::SerializationError)?;
    
    trading_vault_account.data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    
    // Emit事件
    msg!("POSITION_CLOSED|account:{}|market:{}|close_size:{}|exit_price:{}|pnl:{}|returned:{}",
        account_id, market, close_size_e6, exit_price_e6, realized_pnl, actual_return);
    
    msg!("✅ Position closed successfully!");
    msg!("  Returned: {} USDC (e6)", actual_return);
    
    Ok(())
}

/// 强平
fn process_liquidate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    account_id: String,
    market: String,
    liquidation_price_e6: i64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let user_position_account = next_account_info(account_iter)?;
    let position_owner = next_account_info(account_iter)?;
    let liquidator = next_account_info(account_iter)?;
    let liquidator_usdc_account = next_account_info(account_iter)?;
    let user_usdc_account = next_account_info(account_iter)?;
    let vault_usdc_account = next_account_info(account_iter)?;
    let insurance_fund_account = next_account_info(account_iter)?;
    let fee_treasury_account = next_account_info(account_iter)?;
    let trading_vault_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    
    msg!("Liquidate - Processing");
    msg!("  Account: {}", account_id);
    msg!("  Market: {}", market);
    msg!("  Liquidation Price: {}", liquidation_price_e6);
    
    // 验证liquidator签名
    if !liquidator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // 验证Position PDA
    let (expected_pda, _) = Pubkey::find_program_address(
        &[
            b"position",
            position_owner.key.as_ref(),
            account_id.as_bytes(),
            market.as_bytes(),
        ],
        program_id,
    );
    
    if user_position_account.key != &expected_pda {
        msg!("Error: Position PDA mismatch");
        return Err(TradingError::InvalidPositionAccount.into());
    }
    
    // 读取持仓
    let mut position = UserPosition::try_from_slice(&user_position_account.data.borrow())
        .map_err(|_| TradingError::DeserializationError)?;
    
    // 验证持仓归属
    if position.wallet != *position_owner.key {
        return Err(TradingError::InvalidPositionAccount.into());
    }
    
    // 更新PnL（使用清算价格）
    position.update_pnl(liquidation_price_e6);
    
    msg!("  Margin Ratio: {}bp", position.margin_ratio_bp);
    msg!("  Unrealized PnL: {}", position.unrealized_pnl_e6);
    
    // 验证可强平（保证金率 < 100%）
    if position.margin_ratio_bp >= 10000 {
        msg!("Error: Position not liquidatable, margin_ratio = {}bp", position.margin_ratio_bp);
        return Err(TradingError::PositionNotLiquidatable.into());
    }
    
    // 计算清算相关金额
    let equity = position.locked_usdc_e6 + position.unrealized_pnl_e6;
    let locked_usdc = position.locked_usdc_e6;
    
    // 清算手续费：0.5% of locked_usdc
    let liquidation_fee = calculate_liquidation_fee(locked_usdc);
    
    msg!("  Equity: {} USDC", equity as f64 / 1_000_000.0);
    msg!("  Liquidation Fee: {} USDC", liquidation_fee as f64 / 1_000_000.0);
    
    // 派生Vault PDA以签名
    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[b"trading_vault"],
        program_id,
    );
    
    // 分配资金
    let mut total_transferred: i64 = 0;
    
    // 1. 清算手续费（50%给清算人，50%给Fee Treasury）
    if liquidation_fee > 0 {
        let liquidator_fee = liquidation_fee / 2;
        let treasury_fee = liquidation_fee - liquidator_fee;
        
        // 给清算人
        if liquidator_fee > 0 {
            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program.key,
                    vault_usdc_account.key,
                    liquidator_usdc_account.key,
                    &vault_pda,
                    &[],
                    liquidator_fee as u64,
                )?,
                &[
                    vault_usdc_account.clone(),
                    liquidator_usdc_account.clone(),
                    trading_vault_account.clone(),
                    token_program.clone(),
                ],
                &[&[b"trading_vault", &[vault_bump]]],
            )?;
            total_transferred = safe_add_i64(total_transferred, liquidator_fee)?;
            msg!("  Liquidator Fee: {} USDC", liquidator_fee as f64 / 1_000_000.0);
        }
        
        // 给Fee Treasury
        if treasury_fee > 0 {
            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program.key,
                    vault_usdc_account.key,
                    fee_treasury_account.key,
                    &vault_pda,
                    &[],
                    treasury_fee as u64,
                )?,
                &[
                    vault_usdc_account.clone(),
                    fee_treasury_account.clone(),
                    trading_vault_account.clone(),
                    token_program.clone(),
                ],
                &[&[b"trading_vault", &[vault_bump]]],
            )?;
            total_transferred = safe_add_i64(total_transferred, treasury_fee)?;
            msg!("  Fee Treasury: {} USDC", treasury_fee as f64 / 1_000_000.0);
        }
    }
    
    // 2. 剩余资金处理
    let remaining = safe_sub_i64(locked_usdc, total_transferred)?;
    
    if equity > 0 && remaining > 0 {
        // 有剩余返还给用户
        let user_return = remaining.min(equity);
        
        if user_return > 0 {
            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program.key,
                    vault_usdc_account.key,
                    user_usdc_account.key,
                    &vault_pda,
                    &[],
                    user_return as u64,
                )?,
                &[
                    vault_usdc_account.clone(),
                    user_usdc_account.clone(),
                    trading_vault_account.clone(),
                    token_program.clone(),
                ],
                &[&[b"trading_vault", &[vault_bump]]],
            )?;
            total_transferred = safe_add_i64(total_transferred, user_return)?;
            msg!("  User Returned: {} USDC", user_return as f64 / 1_000_000.0);
        }
    }
    
    // 3. 剩余进Insurance Fund（如果还有）
    let final_remaining = safe_sub_i64(locked_usdc, total_transferred)?;
    
    if final_remaining > 0 {
        invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                vault_usdc_account.key,
                insurance_fund_account.key,
                &vault_pda,
                &[],
                final_remaining as u64,
            )?,
            &[
                vault_usdc_account.clone(),
                insurance_fund_account.clone(),
                trading_vault_account.clone(),
                token_program.clone(),
            ],
            &[&[b"trading_vault", &[vault_bump]]],
        )?;
        msg!("  Insurance Fund: {} USDC", final_remaining as f64 / 1_000_000.0);
    }
    
    // 更新Trading Vault
    let mut vault = TradingVault::try_from_slice(&trading_vault_account.data.borrow())
        .map_err(|_| TradingError::DeserializationError)?;
    
    vault.total_locked_usdc_e6 = safe_sub_i64(vault.total_locked_usdc_e6, locked_usdc)?;
    vault.total_positions = vault.total_positions.saturating_sub(1);
    vault.total_liquidations += 1;
    
    if final_remaining > 0 {
        vault.insurance_fund_e6 = safe_add_i64(vault.insurance_fund_e6, final_remaining)?;
    }
    
    vault.updated_at = solana_program::clock::Clock::get()?.unix_timestamp;
    
    let serialized = vault.try_to_vec()
        .map_err(|_| TradingError::SerializationError)?;
    
    trading_vault_account.data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    
    // 删除Position PDA（TODO: 实际应该close account并返还租金）
    // 暂时清零数据
    user_position_account.data.borrow_mut().fill(0);
    
    // Emit事件
    msg!("LIQUIDATION|account:{}|market:{}|liquidation_price:{}|loss:{}|liquidation_fee:{}|equity:{}",
        account_id, market, liquidation_price_e6, 
        locked_usdc - equity, liquidation_fee, equity);
    
    msg!("✅ Position liquidated successfully!");
    msg!("  Total locked: {} USDC", locked_usdc as f64 / 1_000_000.0);
    msg!("  Liquidation fee: {} USDC", liquidation_fee as f64 / 1_000_000.0);
    msg!("  Loss: {} USDC", (locked_usdc - equity) as f64 / 1_000_000.0);
    
    Ok(())
}

/// Smart Hedge部分平仓
fn process_partial_close_for_hedge(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _account_id: String,
    _market: String,
    _close_ratio: u32,
    _exit_price_e6: i64,
    _hedge_mode: crate::state::HedgeMode,
) -> ProgramResult {
    // TODO: 实现Smart Hedge逻辑
    msg!("PartialCloseForHedge - To be implemented");
    Ok(())
}

/// 创建反向建仓
fn process_create_reentry_position(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _account_id: String,
    _market: String,
    _entry_price_e6: i64,
    _pool_created_at: i64,
) -> ProgramResult {
    // TODO: 实现反向建仓逻辑
    msg!("CreateReentryPosition - To be implemented");
    Ok(())
}

/// 执行止盈止损
fn process_execute_tpsl(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _account_id: String,
    _market: String,
    _exit_price_e6: i64,
    _is_take_profit: bool,
    _pool_created_at: i64,
) -> ProgramResult {
    // TODO: 实现止盈止损逻辑
    msg!("ExecuteTpSl - To be implemented");
    Ok(())
}

/// 更新持仓（链下定期调用）
fn process_update_position(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    account_id: String,
    market: String,
    wallet: Pubkey,
    mark_price_e6: i64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let user_position_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // 验证authority
    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // 验证PDA
    let (expected_pda, _) = Pubkey::find_program_address(
        &[
            b"position",
            wallet.as_ref(),
            account_id.as_bytes(),
            market.as_bytes(),
        ],
        program_id,
    );
    
    if user_position_account.key != &expected_pda {
        return Err(TradingError::InvalidPositionAccount.into());
    }
    
    // 读取并更新持仓
    let mut position = UserPosition::try_from_slice(&user_position_account.data.borrow())
        .map_err(|_| TradingError::DeserializationError)?;
    
    position.update_pnl(mark_price_e6);
    
    let serialized = position.try_to_vec()
        .map_err(|_| TradingError::SerializationError)?;
    
    user_position_account.data.borrow_mut()[..serialized.len()]
        .copy_from_slice(&serialized);
    
    msg!("Position updated: mark_price={}, unrealized_pnl={}, margin_ratio={}bp",
        mark_price_e6, position.unrealized_pnl_e6, position.margin_ratio_bp);
    
    Ok(())
}

/// 提现Insurance Fund
fn process_withdraw_insurance_fund(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _amount_e6: i64,
) -> ProgramResult {
    // TODO: 实现Insurance Fund提现逻辑
    msg!("WithdrawInsuranceFund - To be implemented");
    Ok(())
}

