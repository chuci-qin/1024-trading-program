//! Liquidation（强平）集成测试

use trading_program::utils::*;
use trading_program::state::UserPosition;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_liquidation_fee_calculation() {
        // 测试清算手续费计算（0.5% = 50 bps）
        
        let locked_usdc = 10_000_000; // $10
        let fee = calculate_liquidation_fee(locked_usdc);
        
        // Fee = 10 × 0.5% = $0.05
        assert_eq!(fee, 50_000);
        
        // 大额测试
        let locked_usdc_large = 1_000_000_000; // $1,000
        let fee_large = calculate_liquidation_fee(locked_usdc_large);
        assert_eq!(fee_large, 5_000_000); // $5
        
        println!("✅ Liquidation fee calculation test passed");
    }
    
    #[test]
    fn test_liquidation_condition_check() {
        // 测试强平条件判断
        
        // 保证金率 > 100%：不可强平
        assert!(!is_liquidatable(15000)); // 150%
        assert!(!is_liquidatable(12000)); // 120%
        assert!(!is_liquidatable(10001)); // 100.01%
        assert!(!is_liquidatable(10000)); // 100%（边界，不强平）
        
        // 保证金率 < 100%：可强平
        assert!(is_liquidatable(9999));  // 99.99% ✅
        assert!(is_liquidatable(8000));  // 80% ✅
        assert!(is_liquidatable(5000));  // 50% ✅
        assert!(is_liquidatable(0));     // 0% ✅
        
        println!("✅ Liquidation condition check test passed");
    }
    
    #[test]
    fn test_liquidation_loss_scenarios() {
        // 场景1: 保证金率90%，强平后有剩余
        let im = 10_000_000; // $10
        let unrealized_pnl = -1_000_000; // -$1
        let equity = im + unrealized_pnl; // $9
        let mm = 5_000_000; // $5
        let margin_ratio_bp = (equity as i128 * 10000 / mm as i128) as u32;
        
        assert_eq!(margin_ratio_bp, 18000); // 180%，实际不会被强平
        
        // 场景2: 保证金率90%（应该可强平）
        let unrealized_pnl_2 = -5_500_000; // -$5.5
        let equity_2 = im + unrealized_pnl_2; // $4.5
        let margin_ratio_bp_2 = (equity_2 as i128 * 10000 / mm as i128) as u32;
        
        assert_eq!(margin_ratio_bp_2, 9000); // 90% < 100% ✅
        assert!(is_liquidatable(margin_ratio_bp_2));
        
        // 验证清算费
        let liquidation_fee = calculate_liquidation_fee(im);
        assert_eq!(liquidation_fee, 50_000); // $0.05
        
        // 用户应得返还
        let user_return = equity_2 - liquidation_fee as i64;
        assert_eq!(user_return, 4_450_000); // $4.45
        
        println!("✅ Liquidation loss scenario test passed");
    }
    
    #[test]
    fn test_liquidation_total_loss_scenario() {
        // 场景: 保证金率 < 0%（完全亏损）
        let im = 10_000_000; // $10
        let unrealized_pnl = -12_000_000; // -$12
        let equity = im + unrealized_pnl; // -$2
        let mm = 5_000_000; // $5
        
        // 保证金率为负
        let margin_ratio_bp = if equity <= 0 {
            0
        } else {
            (equity as i128 * 10000 / mm as i128) as u32
        };
        
        assert_eq!(margin_ratio_bp, 0); // 0%
        assert!(is_liquidatable(margin_ratio_bp));
        
        // 用户无返还
        let user_return = if equity > 0 { equity } else { 0 };
        assert_eq!(user_return, 0);
        
        // 清算人收费
        let liquidation_fee = calculate_liquidation_fee(im);
        
        // 损失进Insurance Fund
        let loss_to_insurance = if equity < 0 { -equity } else { 0 };
        assert_eq!(loss_to_insurance, 2_000_000); // $2损失
        
        println!("✅ Liquidation total loss scenario test passed");
    }
    
    #[test]
    fn test_liquidation_edge_cases() {
        // 边界测试
        
        // 1. 保证金率刚好100%
        assert!(!is_liquidatable(10000));
        
        // 2. 保证金率99.99%
        assert!(is_liquidatable(9999));
        
        // 3. 极小保证金率
        assert!(is_liquidatable(1));
        
        // 4. 零保证金率
        assert!(is_liquidatable(0));
        
        // 5. 极大杠杆下的强平
        let size_e6 = 1_000_000; // 0.001 BTC
        let price_e6 = 100_000_000_000; // $100,000
        let leverage = 100; // 100x
        
        let im = calculate_initial_margin(size_e6, price_e6, leverage).unwrap();
        assert_eq!(im, 1_000_000_000); // $1 IM (100x杠杆)
        
        // 价格跌1%就强平
        let new_price = 99_000_000_000; // $99,000 (-1%)
        let unrealized_pnl = calculate_realized_pnl(true, price_e6, new_price, size_e6).unwrap();
        assert_eq!(unrealized_pnl, -1_000_000_000); // -$1
        
        let equity = im as i64 + unrealized_pnl;
        assert_eq!(equity, 0); // 刚好爆仓
        
        println!("✅ Liquidation edge cases test passed");
    }
}

