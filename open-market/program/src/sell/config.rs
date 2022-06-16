use anchor_lang::prelude::*;
use solana_program::clock::UnixTimestamp;

pub const BID_SIZE: usize = 8 + 1 + 32;
pub const LISTING_CONFIG_SIZE: usize = 8 + 1 + 8 + 8 + BID_SIZE + 32 + 8 + 1;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub enum ListingConfigVersion {
    V0,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct Bid {
    pub version: ListingConfigVersion,
    pub amount: u64,
    pub buyer_trade_state: Pubkey,
}

#[account]
pub struct ListingConfig {
    pub version: ListingConfigVersion,
    pub start_time: UnixTimestamp,
    pub end_time: UnixTimestamp,
    pub highest_bid: Bid,
    pub listing_auction_house: Pubkey,
    pub listing_fee: u64,
    pub bump: u8,
}
