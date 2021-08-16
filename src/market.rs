use std::{borrow::Cow, convert::identity};

use safe_transmute::{transmute_many_pedantic, transmute_one_pedantic, transmute_one_to_bytes, transmute_to_bytes};
use serum_dex::state::{
    gen_vault_signer_key, AccountFlag, Market, MarketState, MarketStateV2, ACCOUNT_HEAD_PADDING, ACCOUNT_TAIL_PADDING,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Solana client error: {0}")]
    ClientError(#[from] solana_client::client_error::ClientError),

    #[error("Dex error: {0}")]
    DexError(#[from] serum_dex::error::DexError),

    #[error("Program error: {0}")]
    ProgramError(#[from] solana_sdk::program_error::ProgramError),

    #[error("Dex account length {0} is too small to contain valid padding")]
    AccountLengthTooSmall(usize),

    #[error("Dex account head padding mismatch")]
    HeadPaddingMismatch,

    #[error("Dex account tail padding mismatch")]
    TailPaddingMismatch,

    #[error("The transmute data does not respect the target type's boundaries: {0:?}")]
    TransmuteGuard(safe_transmute::GuardError),

    #[error("The transmute data contains an invalid value for the target type")]
    TransmuteInvalidValue,

    #[error("Transmute error: {0}")]
    TransmuteOther(String),
}

impl<'a, T, G> From<safe_transmute::Error<'a, T, G>> for Error {
    fn from(transmute_err: safe_transmute::Error<'a, T, G>) -> Self {
        match transmute_err {
            safe_transmute::Error::Guard(guard) => Self::TransmuteGuard(guard),
            safe_transmute::Error::InvalidValue => Self::TransmuteInvalidValue,
            err => Self::TransmuteOther(format!("{:?}", err)),
        }
    }
}

#[derive(Debug)]
pub struct MarketPubkeys {
    pub market: Pubkey,
    pub req_q: Pubkey,
    pub event_q: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub coin_mint: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_mint: Pubkey,
    pub pc_vault: Pubkey,
    pub vault_signer_key: Pubkey,
}

#[cfg(target_endian = "little")]
pub fn get_market_keys(client: &RpcClient, dex_program_id: Pubkey, market: Pubkey) -> Result<MarketPubkeys, Error> {
    let account_data = client.get_account_data(&market)?;
    let words = remove_dex_account_padding(&account_data)?;
    let market_state = {
        let account_flags = Market::account_flags(&account_data)?;
        if account_flags.intersects(AccountFlag::Permissioned) {
            let state =
                transmute_one_pedantic::<MarketStateV2>(transmute_to_bytes(&words)).map_err(|err| err.without_src())?;
            state.inner
        } else {
            transmute_one_pedantic::<MarketState>(transmute_to_bytes(&words)).map_err(|err| err.without_src())?
        }
    };
    market_state.check_flags()?;

    let vault_signer_key = gen_vault_signer_key(market_state.vault_signer_nonce, &market, &dex_program_id)?;
    assert_eq!(transmute_to_bytes(&identity(market_state.own_address)), market.as_ref());

    Ok(MarketPubkeys {
        market,
        req_q: Pubkey::new(transmute_one_to_bytes(&identity(market_state.req_q))),
        event_q: Pubkey::new(transmute_one_to_bytes(&identity(market_state.event_q))),
        bids: Pubkey::new(transmute_one_to_bytes(&identity(market_state.bids))),
        asks: Pubkey::new(transmute_one_to_bytes(&identity(market_state.asks))),
        coin_mint: Pubkey::new(transmute_one_to_bytes(&identity(market_state.coin_mint))),
        coin_vault: Pubkey::new(transmute_one_to_bytes(&identity(market_state.coin_vault))),
        pc_mint: Pubkey::new(transmute_one_to_bytes(&identity(market_state.pc_mint))),
        pc_vault: Pubkey::new(transmute_one_to_bytes(&identity(market_state.pc_vault))),
        vault_signer_key,
    })
}

fn remove_dex_account_padding(data: &[u8]) -> Result<Cow<[u64]>, Error> {
    let head = &data[..ACCOUNT_HEAD_PADDING.len()];
    if data.len() < ACCOUNT_HEAD_PADDING.len() + ACCOUNT_TAIL_PADDING.len() {
        return Err(Error::AccountLengthTooSmall(data.len()));
    }
    if head != ACCOUNT_HEAD_PADDING {
        return Err(Error::HeadPaddingMismatch);
    }
    let tail = &data[data.len() - ACCOUNT_TAIL_PADDING.len()..];
    if tail != ACCOUNT_TAIL_PADDING {
        return Err(Error::TailPaddingMismatch);
    }
    let inner = &data[ACCOUNT_HEAD_PADDING.len()..(data.len() - ACCOUNT_TAIL_PADDING.len())];
    let words = match transmute_many_pedantic::<u64>(inner) {
        Ok(word_slice) => Cow::Borrowed(word_slice),
        Err(transmute_error) => {
            let word_vec = transmute_error.copy().map_err(|err| err.without_src())?;
            Cow::Owned(word_vec)
        },
    };
    Ok(words)
}
