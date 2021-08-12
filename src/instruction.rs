use borsh::ser::BorshSerialize;
use serum_swap::{ExchangeRate, Side, instruction};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

pub fn swap(
    swap_program_id: Pubkey,
    dex_program_id: Pubkey,
    authority: Pubkey,
    pc_wallet: Pubkey,
    market: Pubkey,
    open_orders: Pubkey,
    request_queue: Pubkey,
    event_queue: Pubkey,
    bids: Pubkey,
    asks: Pubkey,
    order_payer_token_account: Pubkey,
    coin_vault: Pubkey,
    pc_vault: Pubkey,
    vault_signer: Pubkey,
    coin_wallet: Pubkey,
    amount: u64,
    side: Side,
    rate: u64,
    from_decimals: u8,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(market, false),
        AccountMeta::new(open_orders, false),
        AccountMeta::new(request_queue, false),
        AccountMeta::new(event_queue, false),
        AccountMeta::new(bids, false),
        AccountMeta::new(asks, false),
        AccountMeta::new(order_payer_token_account, false),
        AccountMeta::new(coin_vault, false),
        AccountMeta::new(pc_vault, false),
        AccountMeta::new_readonly(vault_signer, false),
        AccountMeta::new(coin_wallet, false),
        AccountMeta::new_readonly(authority, true),
        AccountMeta::new(pc_wallet, false),
        AccountMeta::new_readonly(dex_program_id, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Ok(Instruction {
        program_id: swap_program_id,
        accounts,
        data: instruction::Swap {
            side,
            amount,
            min_exchange_rate: ExchangeRate {
                rate,
                from_decimals,
                quote_decimals: 0,
                strict: false,
            },
        }
            .try_to_vec()?,
    })
}