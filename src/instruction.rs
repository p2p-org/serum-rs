use anchor_lang::InstructionData;
use serum_swap::{instruction, ExchangeRate, Side};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

#[derive(Debug, Clone)]
pub struct MarketAccounts<'a> {
    pub market: &'a Pubkey,
    pub open_orders: &'a Pubkey,
    pub request_queue: &'a Pubkey,
    pub event_queue: &'a Pubkey,
    pub bids: &'a Pubkey,
    pub asks: &'a Pubkey,

    /// The `spl_token::Account` that funds will be taken from, i.e., transferred
    /// from the user into the market's vault.
    ///
    /// For bids, this is the base currency. For asks, the quote.
    pub order_payer_token_account: &'a Pubkey,

    /// Also known as the "base" currency. For a given A/B market,
    /// this is the vault for the A mint.
    pub coin_vault: &'a Pubkey,

    /// Also known as the "quote" currency. For a given A/B market,
    /// this is the vault for the B mint.
    pub pc_vault: &'a Pubkey,

    /// PDA owner of the DEX's token accounts for base + quote currencies.
    pub vault_signer: &'a Pubkey,

    /// User wallets.
    pub coin_wallet: &'a Pubkey,
}

pub fn init_account(
    swap_program_id: Pubkey,
    dex_program_id: Pubkey,
    authority: Pubkey,
    market: Pubkey,
    open_orders: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(open_orders, false),
        AccountMeta::new_readonly(authority, true),
        AccountMeta::new_readonly(market, false),
        AccountMeta::new_readonly(dex_program_id, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Instruction {
        program_id: swap_program_id,
        accounts,
        data: instruction::InitAccount.data(),
    }
}

pub fn swap(
    swap_program_id: &Pubkey,
    dex_program_id: &Pubkey,
    authority: &Pubkey,
    pc_wallet: &Pubkey,
    market: &MarketAccounts,
    amount: u64,
    side: Side,
    rate: u64,
    from_decimals: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*market.market, false),
        AccountMeta::new(*market.open_orders, false),
        AccountMeta::new(*market.request_queue, false),
        AccountMeta::new(*market.event_queue, false),
        AccountMeta::new(*market.bids, false),
        AccountMeta::new(*market.asks, false),
        AccountMeta::new(*market.order_payer_token_account, false),
        AccountMeta::new(*market.coin_vault, false),
        AccountMeta::new(*market.pc_vault, false),
        AccountMeta::new_readonly(*market.vault_signer, false),
        AccountMeta::new(*market.coin_wallet, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*pc_wallet, false),
        AccountMeta::new_readonly(*dex_program_id, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Instruction {
        program_id: *swap_program_id,
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
        .data(),
    }
}

pub fn swap_transitive(
    swap_program_id: &Pubkey,
    dex_program_id: &Pubkey,
    authority: &Pubkey,
    pc_wallet: &Pubkey,
    from: &MarketAccounts,
    to: &MarketAccounts,
    amount: u64,
    rate: u64,
    from_decimals: u8,
    quote_decimals: u8,
    strict: bool,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*from.market, false),
        AccountMeta::new(*from.open_orders, false),
        AccountMeta::new(*from.request_queue, false),
        AccountMeta::new(*from.event_queue, false),
        AccountMeta::new(*from.bids, false),
        AccountMeta::new(*from.asks, false),
        AccountMeta::new(*from.order_payer_token_account, false),
        AccountMeta::new(*from.coin_vault, false),
        AccountMeta::new(*from.pc_vault, false),
        AccountMeta::new_readonly(*from.vault_signer, false),
        AccountMeta::new(*from.coin_wallet, false),
        AccountMeta::new(*to.market, false),
        AccountMeta::new(*to.open_orders, false),
        AccountMeta::new(*to.request_queue, false),
        AccountMeta::new(*to.event_queue, false),
        AccountMeta::new(*to.bids, false),
        AccountMeta::new(*to.asks, false),
        AccountMeta::new(*to.order_payer_token_account, false),
        AccountMeta::new(*to.coin_vault, false),
        AccountMeta::new(*to.pc_vault, false),
        AccountMeta::new_readonly(*to.vault_signer, false),
        AccountMeta::new(*to.coin_wallet, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*pc_wallet, false),
        AccountMeta::new_readonly(*dex_program_id, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Instruction {
        program_id: *swap_program_id,
        accounts,
        data: instruction::SwapTransitive {
            amount,
            min_exchange_rate: ExchangeRate {
                rate,
                from_decimals,
                quote_decimals,
                strict,
            },
        }
        .data(),
    }
}

pub fn close_account(
    swap_program_id: Pubkey,
    dex_program_id: Pubkey,
    authority: Pubkey,
    market: Pubkey,
    open_orders: Pubkey,
    destination: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(open_orders, false),
        AccountMeta::new_readonly(authority, true),
        AccountMeta::new(destination, false),
        AccountMeta::new_readonly(market, false),
        AccountMeta::new_readonly(dex_program_id, false),
    ];
    Instruction {
        program_id: swap_program_id,
        accounts,
        data: instruction::CloseAccount.data(),
    }
}

mod tests {
    pub use super::*;

    #[test]
    fn serialize_instruction() {
        let pubkey = Pubkey::new_unique();
        let init = init_account(pubkey, pubkey, pubkey, pubkey, pubkey);
        let swap = swap(
            &pubkey,
            &pubkey,
            &pubkey,
            &pubkey,
            &MarketAccounts {
                market: &pubkey,
                open_orders: &pubkey,
                request_queue: &pubkey,
                event_queue: &pubkey,
                bids: &pubkey,
                asks: &pubkey,
                order_payer_token_account: &pubkey,
                coin_vault: &pubkey,
                pc_vault: &pubkey,
                vault_signer: &pubkey,
                coin_wallet: &pubkey,
            },
            1,
            Side::Ask,
            2,
            3,
        );

        assert_eq!(init.data, vec![169, 188, 158, 199, 9, 151, 101, 125]);
        assert_eq!(swap.data, vec![
            248, 198, 158, 145, 225, 117, 135, 200, 1, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0
        ]);
    }
}
