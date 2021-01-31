use sp_std::prelude::*;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

/// Rtoken Identifier
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum OpposeReason {
	/// blockhash
	BLOCKHASH,
	/// txhash
    TXHASH,
    /// from
    FROM,
    /// to
    TO,
    /// amount
    AMOUNT(u128, u128),
}

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum BondStatus {
    Initiated,
    Approved,
    Rejected,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct BondRecord<AccountId, Symbol> {
    pub bonder: AccountId,
    pub symbol: Symbol,
    pub blockhash: Vec<u8>,
    pub txhash: Vec<u8>,
    pub amount: u128,
}

impl<A: PartialEq, B: PartialEq> BondRecord<A, B> {
    pub fn new(boonder: A, symbol: B, blockhash: Vec<u8>, txhash: Vec<u8>, amount: u128) -> Self {
        Self {
            bonder: boonder,
            symbol: symbol,
            blockhash: blockhash,
            txhash: txhash,
            amount: amount,
        }
    }
}



#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct BondVote<Hash, AccountId, BlockNumber> {
    pub id: Hash,
    pub votes_for: Vec<AccountId>,
    pub votes_against: Vec<AccountId>,
    pub against_reasons: Vec<OpposeReason>,
    pub status: BondStatus,
    pub expiry: BlockNumber,
}

impl<A: PartialEq, B: PartialEq, C: PartialOrd + Default> BondVote<A, B, C> {
    /// derivate next status according to threshold and now
    pub fn derivate(&mut self, threshold: u32, total: u32) {
        if self.is_completed() {return}
        if self.votes_for.len() >= threshold as usize {
            self.status = BondStatus::Approved;
        } else if total >= threshold && self.votes_against.len() as u32 + threshold > total {
            self.status = BondStatus::Rejected;
        }
    }

    /// Returns true if the proposal has been rejected or approved, otherwise false.
    pub fn is_completed(&self) -> bool {
        self.status != BondStatus::Initiated
    }

    /// Returns true if `who` has voted for or against the proposal
    pub fn has_voted(&self, who: &B) -> bool {
        self.votes_for.contains(&who) || self.votes_against.contains(&who)
    }

    /// Return true if the expiry time has been reached
    pub fn is_expired(&self, now: C) -> bool {
        self.expiry <= now
    }

    pub fn is_approved(&self) -> bool {
        self.status == BondStatus::Approved
    }

    pub fn is_rejected(&self) -> bool {
        self.status == BondStatus::Rejected
    }

    pub fn set_expiry(&mut self, expiry: C) {
        self.expiry = expiry;
    }

    pub fn new(id: A, expiry: C) -> Self {
        Self {
            id: id,
            votes_for: vec![],
            votes_against: vec![],
            against_reasons: vec![],
            status: BondStatus::Initiated,
            expiry: expiry,
        }
    }
}