//! Trading Program Utility Functions

use crate::error::TradingError;
use solana_program::program_error::ProgramError;

/// 验证杠杆范围（1-100x）
pub fn validate_leverage(leverage: u32) -> Result<(), ProgramError> {
    if leverage == 0 || leverage > 100 {
        return Err(TradingError::InvalidLeverage.into());
    }
    Ok(())
}

/// 验证价格（必须>0）
pub fn validate_price(price_e6: i64) -> Result<(), ProgramError> {
    if price_e6 <= 0 {
        return Err(TradingError::InvalidPrice.into());
    }
    Ok(())
}

/// 验证数量（必须>0）
pub fn validate_size(size_e6: i64) -> Result<(), ProgramError> {
    if size_e6 <= 0 {
        return Err(TradingError::InvalidSize.into());
    }
    Ok(())
}

/// 验证市场名称
pub fn validate_market(market: &str) -> Result<(), ProgramError> {
    const VALID_MARKETS: &[&str] = &["BTC-PERP", "ETH-PERP", "SOL-PERP"];
    
    if !VALID_MARKETS.contains(&market) {
        return Err(TradingError::InvalidMarket.into());
    }
    Ok(())
}

/// 验证账户ID长度
pub fn validate_account_id(account_id: &str) -> Result<(), ProgramError> {
    if account_id.is_empty() || account_id.len() > 64 {
        return Err(TradingError::InvalidAccountId.into());
    }
    Ok(())
}

/// 计算初始保证金（IM）
/// IM = notional / leverage
/// notional = size × entry_price
pub fn calculate_initial_margin(
    size_e6: i64,
    entry_price_e6: i64,
    leverage: u32,
) -> Result<i64, ProgramError> {
    // 防止溢出，使用i128
    let notional = (size_e6 as i128)
        .checked_mul(entry_price_e6 as i128)
        .ok_or(TradingError::ArithmeticOverflow)?
        / 1_000_000;
    
    let im = notional
        .checked_div(leverage as i128)
        .ok_or(TradingError::DivisionByZero)?;
    
    Ok(im as i64)
}

/// 计算维持保证金（MM）
/// MM = IM / 2
pub fn calculate_maintenance_margin(im_e6: i64) -> i64 {
    im_e6 / 2
}

/// 计算已实现盈亏
/// PnL = (exit_price - entry_price) × close_size (for Long)
/// PnL = (entry_price - exit_price) × close_size (for Short)
pub fn calculate_realized_pnl(
    is_long: bool,
    entry_price_e6: i64,
    exit_price_e6: i64,
    close_size_e6: i64,
) -> Result<i64, ProgramError> {
    let price_diff = if is_long {
        exit_price_e6 - entry_price_e6
    } else {
        entry_price_e6 - exit_price_e6
    };
    
    // PnL = price_diff × size (e6格式)
    let pnl = (price_diff as i128)
        .checked_mul(close_size_e6 as i128)
        .ok_or(TradingError::ArithmeticOverflow)?
        / 1_000_000;
    
    Ok(pnl as i64)
}

// === Smart Hedge费用计算已移除 ===
// Smart Hedge应该在1024-core/smart-hedge-engine中实现

/// 计算清算手续费（0.5%）
pub fn calculate_liquidation_fee(locked_usdc_e6: i64) -> i64 {
    // 0.5% = 0.005
    (locked_usdc_e6 as i128 * 5 / 1000) as i64
}

/// 安全的i64加法
pub fn safe_add_i64(a: i64, b: i64) -> Result<i64, ProgramError> {
    a.checked_add(b).ok_or(TradingError::ArithmeticOverflow.into())
}

/// 安全的i64减法
pub fn safe_sub_i64(a: i64, b: i64) -> Result<i64, ProgramError> {
    a.checked_sub(b).ok_or(TradingError::ArithmeticUnderflow.into())
}

/// 验证保证金率是否可强平（<100%）
pub fn is_liquidatable(margin_ratio_bp: u32) -> bool {
    margin_ratio_bp < 10000 // 100% = 10000 bp
}

// === Smart Hedge触发判断已移除 ===
// Smart Hedge应该在1024-core/smart-hedge-engine中实现

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_initial_margin() {
        // 0.001 BTC @ $101,885, 20x leverage
        let size_e6 = 1_000_000; // 0.001 BTC (e6)
        let price_e6 = 101_885_000_000; // $101,885 (e6)
        let leverage = 20;
        
        let im = calculate_initial_margin(size_e6, price_e6, leverage).unwrap();
        assert_eq!(im, 5_094_250_000); // $5,094.25 (e6)
    }
    
    #[test]
    fn test_calculate_realized_pnl() {
        // Long: Entry $101,885, Exit $102,500, Size 0.0005 BTC
        let pnl = calculate_realized_pnl(
            true,
            101_885_000_000,
            102_500_000_000,
            500_000,
        ).unwrap();
        assert_eq!(pnl, 307_500_000); // $307.5 (e6)
    }
    
    #[test]
    fn test_calculate_liquidation_fee() {
        let fee = calculate_liquidation_fee(5_000_000);
        assert_eq!(fee, 25_000); // $0.025 (e6)
    }
}

