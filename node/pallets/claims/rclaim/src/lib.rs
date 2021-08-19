// Copyright 2019-2021 Stafi Protocol.
// This file is part of Stafi.

// Stafi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Stafi.  If not, see <http://www.gnu.org/licenses/>.

//! Module to process claims from rtoken mint.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, ExistenceRequirement::KeepAlive},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use general_signature::{ethereum_verify, to_ascii_hex, SigVerifyResult};
use node_primitives::{Balance, BlockNumber, RSymbol};
use sp_arithmetic::helpers_128bit::multiply_by_rational;
use sp_runtime::traits::SaturatedConversion;
use sp_std::convert::TryInto;
use sp_std::prelude::*;

pub mod models;
pub use models::*;

/// Configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

pub const RATEBASE: u128 = 1_000_000_000_000;
// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as RClaim {
		/// claim infos (account, rsymbol, cycle, mint index)
		pub ClaimInfos get(fn claim_infos): map hasher(blake2_128_concat) (T::AccountId, RSymbol, u32, u64) => Option<ClaimInfo>;
		pub REthClaimInfos get(fn reth_claim_infos): map hasher(blake2_128_concat) (Vec<u8>, u32, u64) => Option<ClaimInfo>;
		/// Proxy accounts for setting fees
		pub REthRewarder get(fn reth_rewarder): Option<T::AccountId>;
		/// MintRewardActs
		pub Acts get(fn acts): map hasher(blake2_128_concat) (RSymbol, u32) => Option<MintRewardAct<BlockNumber, Balance>>;
		pub REthActs get(fn reth_acts): map hasher(blake2_128_concat) u32 => Option<MintRewardAct<BlockNumber, Balance>>;
		/// fund address
		pub FundAddress get(fn fund_address): Option<T::AccountId>;
		/// act latest cycle
		pub ActLatestCycle get(fn act_latest_cycle): map hasher(blake2_128_concat) RSymbol => u32;
		pub REthActLatestCycle get(fn reth_act_latest_cycle): u32;
		/// act current cycle
		pub ActCurrentCycle get(fn act_current_cycle): map hasher(blake2_128_concat) RSymbol => u32;
		pub REthActCurrentCycle get(fn reth_act_current_cycle): u32;
		/// acts that user mint rtoken
		pub UserActs get(fn user_acts): map hasher(blake2_128_concat) (T::AccountId, RSymbol) => Option<Vec<u32>>;
		pub UserREthActs get(fn user_reth_acts): map hasher(blake2_128_concat) Vec<u8> => Option<Vec<u32>>;
		/// user mint count (account, rsymbol, cycle)
		pub UserMintsCount get(fn user_mints_count): map hasher(blake2_128_concat) (T::AccountId, RSymbol, u32) => u64;
		pub UserREthMintsCount get(fn user_reth_mints_count): map hasher(blake2_128_concat) (Vec<u8>, u32) => u64;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// Someone claimed some fis from mint rtoken
		RTokenClaimed(AccountId, RSymbol, u128),
		/// Someone claimed some fis from mint reth
		REthClaimed(AccountId, u128),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Address has no claimInfo.
		HasNoClaimInfo,
		/// hash no act
		HasNoAct,
		/// zero value
		ValueZero,
		/// invalid reth rewarder
		InvalidREthRewarder,
		/// Insufficient fis
		InsufficientFis,
		/// no fund address
		NoFundAddress,
		/// invalid Pubkey
		InvalidPubkey,
		/// eth signature failed
		EthSigsFailed,
		/// pubkey and mint value numnber not equal
		PubkeyAndValueNumberErr,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		type Error = Error<T>;

		// Initializing events
		fn deposit_event() = default;

		/// Set reth rewarder.
		#[weight = 100_000]
		pub fn set_reth_rewarder(origin, account: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			<REthRewarder<T>>::put(account);
			Ok(())
		}

		/// set fund address
		#[weight = 100_000]
		fn set_fund_address(origin, address: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			<FundAddress<T>>::put(address);
			Ok(())
		}


		#[weight = 100_000]
		pub fn add_rtoken_reward_act(
			origin,
			begin: BlockNumber,
			end: BlockNumber,
			symbol: RSymbol,
			total_reward: Balance,
			user_limit: Balance,
			locked_blocks: u32,
			reward_rate: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(begin > 0, "Begin block number must be greater than 0");
			ensure!(end > begin, "End block number must be greater than begin block nubmer");
			ensure!(total_reward > 0, "total amount must be greater than 0");
			ensure!(total_reward > user_limit, "total amount must be greater than User limit");
			ensure!(locked_blocks > 0, "locked blocks must be greater than 0");
			ensure!(reward_rate > 0, "reward rate must be greater than 0");

			let current_block_num = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
			ensure!(end > current_block_num, "End block number must be greater than current block nubmer");

			let cycle = Self::act_latest_cycle(symbol);
			if cycle > 0 {
				let last_act = Self::acts((symbol, cycle)).ok_or(Error::<T>::HasNoAct)?;
				ensure!(begin > last_act.end, "Begin block number must be greater than end block nubmer of the last act");
			}
			let new_cycle = cycle + 1;
			<ActLatestCycle>::insert(symbol, new_cycle);

			let act = MintRewardAct {
				begin: begin,
				end: end,
				cycle: new_cycle,
				reward_rate: reward_rate,
				total_reward: total_reward,
				left_amount: total_reward,
				user_limit: user_limit,
				locked_blocks: locked_blocks,
			};
			<Acts>::insert((symbol, new_cycle), act);
			Ok(())
		}

		#[weight = 100_000]
		pub fn update_rtoken_reward_act(
			origin,
			cycle: u32,
			begin: BlockNumber,
			end: BlockNumber,
			symbol: RSymbol,
			total_reward: Balance,
			user_limit: Balance,
			locked_blocks: u32,
			reward_rate: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(begin > 0, "Begin block number must be greater than 0");
			ensure!(end > begin, "End block number must be greater than begin block nubmer");
			ensure!(total_reward > 0, "total amount must be greater than 0");
			ensure!(total_reward > user_limit, "total amount must be greater than User limit");
			ensure!(locked_blocks > 0, "locked blocks must be greater than 0");
			ensure!(reward_rate > 0, "reward rate must be greater than 0");
			let mut act = Self::acts((symbol, cycle)).ok_or(Error::<T>::HasNoAct)?;

			let current_block_num = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
			ensure!(end > current_block_num, "End block number must be greater than current block nubmer");

			if total_reward > act.total_reward {
				act.left_amount = act.left_amount.saturating_add(total_reward.saturating_sub(act.total_reward));
			} else {
				act.left_amount = act.left_amount.saturating_sub(act.total_reward.saturating_sub(total_reward));
			}

			act.begin = begin;
			act.end = end;
			act.cycle = cycle;
			act.reward_rate = reward_rate;
			act.total_reward = total_reward;
			act.user_limit = user_limit;
			act.locked_blocks = locked_blocks;

			<Acts>::insert((symbol, cycle), act);
			Ok(())
		}

		#[weight = 100_000]
		pub fn add_reth_reward_act(
			origin,
			begin: BlockNumber,
			end: BlockNumber,
			total_reward: Balance,
			user_limit: Balance,
			locked_blocks: u32,
			reward_rate: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(begin > 0, "Begin block number must be greater than 0");
			ensure!(end > begin, "End block number must be greater than begin block nubmer");
			ensure!(total_reward > 0, "total amount must be greater than 0");
			ensure!(total_reward > user_limit, "total amount must be greater than User limit");
			ensure!(locked_blocks > 0, "locked blocks must be greater than 0");
			ensure!(reward_rate > 0, "reward rate must be greater than 0");

			let current_block_num = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
			ensure!(end > current_block_num, "End block number must be greater than current block nubmer");

			let cycle = Self::reth_act_latest_cycle();
			if cycle > 0 {
				let last_act = Self::reth_acts(cycle).ok_or(Error::<T>::HasNoAct)?;
				ensure!(begin > last_act.end, "Begin block number must be greater than end block nubmer of the last act");
			}
			let new_cycle = cycle + 1;
			<REthActLatestCycle>::put(new_cycle);

			let act = MintRewardAct {
				begin: begin,
				end: end,
				cycle: new_cycle,
				reward_rate: reward_rate,
				total_reward: total_reward,
				left_amount: total_reward,
				user_limit: user_limit,
				locked_blocks: locked_blocks,
			};
			<REthActs>::insert(new_cycle, act);

			Ok(())
		}

		#[weight = 100_000]
		pub fn update_reth_reward_act(
			origin,
			cycle: u32,
			begin: BlockNumber,
			end: BlockNumber,
			total_reward: Balance,
			user_limit: Balance,
			locked_blocks: u32,
			reward_rate: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(begin > 0, "Begin block number must be greater than 0");
			ensure!(end > begin, "End block number must be greater than begin block nubmer");
			ensure!(total_reward > 0, "total amount must be greater than 0");
			ensure!(total_reward > user_limit, "total amount must be greater than User limit");
			ensure!(locked_blocks > 0, "locked blocks must be greater than 0");
			ensure!(reward_rate > 0, "reward rate must be greater than 0");
			let mut act = Self::reth_acts(cycle).ok_or(Error::<T>::HasNoAct)?;

			let current_block_num = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
			ensure!(end > current_block_num, "End block number must be greater than current block nubmer");

			if total_reward > act.total_reward {
				act.left_amount = act.left_amount.saturating_add(total_reward.saturating_sub(act.total_reward));
			} else {
				act.left_amount = act.left_amount.saturating_sub(act.total_reward.saturating_sub(total_reward));
			}

			act.begin = begin;
			act.end = end;
			act.cycle = cycle;
			act.reward_rate = reward_rate;
			act.total_reward = total_reward;
			act.user_limit = user_limit;
			act.locked_blocks = locked_blocks;

			<REthActs>::insert(cycle, act);
			Ok(())
		}


		/// Make a rtoken claim
		#[weight = 10_000_000_000]
		pub fn claim_rtoken_reward(origin, symbol: RSymbol, cycle: u32, index: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut claim_info = Self::claim_infos((&who, symbol, cycle, index)).ok_or(Error::<T>::HasNoClaimInfo)?;
			let act = Self::acts((symbol, cycle)).ok_or(Error::<T>::HasNoAct)?;
			let fund_addr = Self::fund_address().ok_or(Error::<T>::NoFundAddress)?;
			let now_block = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
			let final_block = claim_info.mint_block.saturating_add(act.locked_blocks);

			let mut should_claim_amount = claim_info.total_reward.saturating_sub(claim_info.total_claimed);
			if now_block < final_block {
				let du_blocks = now_block.saturating_sub(claim_info.latest_claimed_block) as u128;
				should_claim_amount = multiply_by_rational(claim_info.total_reward, du_blocks, act.locked_blocks as u128).unwrap_or(u128::MIN) as u128;
			}
			ensure!(should_claim_amount > 0, Error::<T>::ValueZero);

			//update state
			T::Currency::transfer(&fund_addr, &who, should_claim_amount.saturated_into(), KeepAlive)?;
			claim_info.total_claimed = claim_info.total_claimed.saturating_add(should_claim_amount);
			claim_info.latest_claimed_block = now_block;
			<ClaimInfos<T>>::insert((who.clone(), symbol, cycle, index), claim_info);

			Self::deposit_event(RawEvent::RTokenClaimed(who.clone(), symbol, should_claim_amount));
			Ok(())
		}

		/// Make a reth claim
		#[weight = 10_000_000_000]
		pub fn claim_reth_reward(origin, pubkey: Vec<u8>, sigs: Vec<u8>, cycle: u32, index: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let use_who = who.using_encoded(to_ascii_hex);
			ensure!(pubkey.len() == 20, Error::<T>::InvalidPubkey);
			ensure!(ethereum_verify(&pubkey, &sigs, &use_who) == SigVerifyResult::Pass, Error::<T>::EthSigsFailed);

			let mut claim_info = Self::reth_claim_infos((pubkey.clone(), cycle, index)).ok_or(Error::<T>::HasNoClaimInfo)?;
			let act = Self::reth_acts(cycle).ok_or(Error::<T>::HasNoAct)?;
			let fund_addr = Self::fund_address().ok_or(Error::<T>::NoFundAddress)?;
			let now_block = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
			let final_block = claim_info.mint_block.saturating_add(act.locked_blocks);

			let mut should_claim_amount = claim_info.total_reward.saturating_sub(claim_info.total_claimed);
			if now_block < final_block {
				let du_blocks = now_block.saturating_sub(claim_info.latest_claimed_block) as u128;
				should_claim_amount = multiply_by_rational(claim_info.total_reward, du_blocks, act.locked_blocks as u128).unwrap_or(u128::MIN) as u128;
			}
			ensure!(should_claim_amount > 0, Error::<T>::ValueZero);

			//update state
			T::Currency::transfer(&fund_addr, &who, should_claim_amount.saturated_into(), KeepAlive)?;
			claim_info.total_claimed = claim_info.total_claimed.saturating_add(should_claim_amount);
			claim_info.latest_claimed_block = now_block;
			<REthClaimInfos>::insert((pubkey.clone(), cycle, index), claim_info);

			Self::deposit_event(RawEvent::REthClaimed(who.clone(), should_claim_amount));
			Ok(())
		}

		#[weight = 100_000]
		pub fn update_reth_claim_info(origin, pubkeys: Vec<Vec<u8>>, mint_values: Vec<u128>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_rewarder(&who), Error::<T>::InvalidREthRewarder);
			ensure!(pubkeys.len() == mint_values.len() && pubkeys.len() < 200, Error::<T>::PubkeyAndValueNumberErr);

			let now_block = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
			let mut cycle = Self::reth_act_current_cycle();
			if cycle == 0 {
				Self::update_reth_act_current_cycle(now_block);
				cycle = Self::reth_act_current_cycle();
				if cycle == 0 {
					return Ok(());
				}
			}
			let act_op = Self::reth_acts(cycle);
			if act_op.is_none() {
				return Ok(());
			}
			let mut act = act_op.unwrap();
			if act.end < now_block {
				Self::update_reth_act_current_cycle(now_block);
				cycle = Self::reth_act_current_cycle();
				let act_op = Self::reth_acts(cycle);
				if act_op.is_none() {
					return Ok(());
				}
				act = act_op.unwrap();
			}
			if act.begin > now_block || act.end < now_block {
				return Ok(());
			}
			if act.left_amount == 0 {
				return Ok(());
			}
			for j in 0..pubkeys.len() {
				ensure!(pubkeys[j].len() == 20, Error::<T>::InvalidPubkey);
				ensure!(mint_values[j] > 0, Error::<T>::ValueZero);
			}
			//update state
			for k in 0..pubkeys.len() {
				if act.left_amount == 0 {
					break;
				}
				let mint_value = mint_values[k];
				let pubkey = &pubkeys[k];
				let mut should_reward_amount = multiply_by_rational(mint_value, act.reward_rate, RATEBASE)
				.unwrap_or(u128::MIN) as u128;
				if should_reward_amount > act.left_amount {
					should_reward_amount = act.left_amount;
				}
				if act.user_limit > 0 && should_reward_amount > act.user_limit {
					should_reward_amount = act.user_limit;
				}
				act.left_amount = act.left_amount.saturating_sub(should_reward_amount);
				let claim_info = ClaimInfo {
					total_reward: should_reward_amount,
					total_claimed: 0,
					latest_claimed_block: now_block,
					mint_block: now_block,
				};
				let mints_count = Self::user_reth_mints_count((pubkey.clone(), cycle));
				//update state
				<REthClaimInfos>::insert((pubkey.clone(), cycle, mints_count), claim_info);
				let mut acts = Self::user_reth_acts(pubkey.clone()).unwrap_or(vec![]);
				if !acts.contains(&cycle) {
					acts.push(cycle);
					<UserREthActs>::insert(pubkey.clone(), acts);
				}
				<UserREthMintsCount>::insert((pubkey.clone(), cycle), mints_count + 1);
			}
			<REthActs>::insert(cycle, act);
			Ok(())
		}


	}
}

impl<T: Trait> Module<T> {
	/// Checks if who is a rewarder
	pub fn is_rewarder(who: &T::AccountId) -> bool {
		let rewarder_op = Self::reth_rewarder();
		if rewarder_op.is_none() {
			return false;
		}
		return &rewarder_op.unwrap() == who;
	}
	/// update user claim info when user mint rtoken
	pub fn update_claim_info(who: &T::AccountId, symbol: RSymbol, mint_value: u128) {
		let mut cycle = Self::act_current_cycle(symbol);
		let now_block = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
		if cycle == 0 {
			Self::update_act_current_cycle(now_block, symbol);
			cycle = Self::act_current_cycle(symbol);
			if cycle == 0 {
				return;
			}
		}
		let act_op = Self::acts((symbol, cycle));
		if act_op.is_none() {
			return;
		}
		let mut act = act_op.unwrap();
		if act.end < now_block {
			Self::update_act_current_cycle(now_block, symbol);
			cycle = Self::act_current_cycle(symbol);
			let act_op = Self::acts((symbol, cycle));
			if act_op.is_none() {
				return;
			}
			act = act_op.unwrap();
		}

		if act.begin > now_block || act.end < now_block {
			return;
		}
		if act.left_amount == 0 {
			return;
		}

		let mut should_reward_amount = multiply_by_rational(mint_value, act.reward_rate, RATEBASE)
			.unwrap_or(u128::MIN) as u128;
		if should_reward_amount > act.left_amount {
			should_reward_amount = act.left_amount;
		}
		if act.user_limit > 0 && should_reward_amount > act.user_limit {
			should_reward_amount = act.user_limit;
		}

		act.left_amount = act.left_amount.saturating_sub(should_reward_amount);

		let claim_info = ClaimInfo {
			total_reward: should_reward_amount,
			total_claimed: 0,
			latest_claimed_block: now_block,
			mint_block: now_block,
		};
		let mints_count = Self::user_mints_count((who, symbol, cycle));

		//update state
		<ClaimInfos<T>>::insert((who, symbol, cycle, mints_count), claim_info);
		let mut acts = Self::user_acts((who, symbol)).unwrap_or(vec![]);
		if !acts.contains(&cycle) {
			acts.push(cycle);
			<UserActs<T>>::insert((who, symbol), acts);
		}
		<UserMintsCount<T>>::insert((who, symbol, cycle), mints_count + 1);
		<Acts>::insert((symbol, cycle), act);
	}

	/// update current act cycle
	fn update_act_current_cycle(now: BlockNumber, symbol: RSymbol) {
		let cycle = Self::act_latest_cycle(symbol);
		if cycle > 0 {
			let last_current_cycle = Self::act_current_cycle(symbol);
			if cycle == last_current_cycle {
				return;
			}

			let begin = last_current_cycle + 1;
			for i in begin..(cycle + 1) {
				let act_op = Self::acts((symbol, i));
				if act_op.is_none() {
					continue;
				}
				let act = act_op.unwrap();
				if now < act.begin {
					break;
				}
				if act.begin <= now && act.end >= now {
					if i != last_current_cycle {
						<ActCurrentCycle>::insert(symbol, i);
					}
					break;
				}
			}
		}
	}

	/// update current act cycle
	fn update_reth_act_current_cycle(now: BlockNumber) {
		let cycle = Self::reth_act_latest_cycle();
		if cycle > 0 {
			let last_current_cycle = Self::reth_act_current_cycle();
			if cycle == last_current_cycle {
				return;
			}

			let begin = last_current_cycle + 1;
			for i in begin..(cycle + 1) {
				let act_op = Self::reth_acts(i);
				if act_op.is_none() {
					continue;
				}
				let act = act_op.unwrap();
				if now < act.begin {
					break;
				}
				if act.begin <= now && act.end >= now {
					if i != last_current_cycle {
						<REthActCurrentCycle>::put(i);
					}
					break;
				}
			}
		}
	}
}
