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

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, ExistenceRequirement::KeepAlive},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use node_primitives::{Balance, BlockNumber, RSymbol};
use rtoken_balances::traits::Currency as RCurrency;
use sp_std::prelude::*;
pub mod models;
pub use models::*;
use pallet_staking::{self as staking};
use sp_arithmetic::helpers_128bit::multiply_by_rational;
use sp_runtime::traits::SaturatedConversion;
use sp_std::convert::TryInto;

/// Configuration trait.
pub trait Trait: system::Trait + staking::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// Currency mechanism of xtoken
	type RCurrency: RCurrency<Self::AccountId>;
}

pub const RATEBASE: u128 = 1_000_000_000_000;
// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as RClaim {
		/// claim infos
		pub ClaimInfos get(fn claim_infos): map hasher(blake2_128_concat) (T::AccountId, RSymbol, u32, Vec<u8>) => Option<ClaimInfo>;
		/// Proxy accounts for setting fees
		pub REthRewarder get(fn reth_rewarder): Option<T::AccountId>;
		/// MintRewardActs
		pub Acts get(fn acts): map hasher(blake2_128_concat) (RSymbol, u32) => Option<MintRewardAct<BlockNumber, Balance>>;
		/// fund address
		pub FundAddress get(fn fund_address): Option<T::AccountId>;
		/// act latest cycle
		pub ActLatestCycle get(fn act_latest_cycle): map hasher(blake2_128_concat) RSymbol => u32;
		/// act current cycle
		pub ActCurrentCycle get(fn act_current_cycle): map hasher(blake2_128_concat) RSymbol => u32;
		/// acts that user mint rtoken
		pub UserActs get(fn user_acts): map hasher(blake2_128_concat) (T::AccountId, RSymbol) => Option<Vec<u32>>;
		/// tx hashs that user mint
		pub UserMints get(fn user_mints): map hasher(blake2_128_concat) (T::AccountId, RSymbol, u32) => Option<Vec<Vec<u8>>>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// Someone claimed some fis
		Claimed(AccountId, RSymbol, u128),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Address has no claimInfo.
		HasNoClaimInfo,
		/// hash no act
		HasNoAct,
		/// There's not enough in the pot to pay out some unvested amount. Generally implies a logic
		/// error.
		PotUnderflow,
		/// zero value
		ValueZero,
		/// invalid reth rewarder
		InvalidREthRewarder,
		/// Got an overflow after adding
		OverFlow,
		/// Insufficient fis
		InsufficientFis,
		NoFundAddress,
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

		fn on_finalize(now: T::BlockNumber) {
			let current_block = now.try_into().ok().unwrap() as BlockNumber;
			Self::update_act_current_cycle(current_block, RSymbol::RDOT);
			Self::update_act_current_cycle(current_block, RSymbol::RFIS);
			Self::update_act_current_cycle(current_block, RSymbol::RKSM);
			Self::update_act_current_cycle(current_block, RSymbol::RATOM);
		}

		/// Set reth rewarder.
		#[weight = 1_000_000]
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
		pub fn add_mint_reward_act(
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

			let current_block_num = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
			ensure!(end > current_block_num, "End block number must be greater than current block nubmer");

			let cycle = Self::act_latest_cycle(symbol);
			if cycle > 0 {
				let last_act = Self::acts((symbol, cycle)).ok_or(Error::<T>::HasNoAct)?;
				ensure!(begin > last_act.end, "Begin block number must be greater than end block nubmer of the last  act");
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

		/// Make a claim
		#[weight = 50_000_000]
		pub fn claim(origin, symbol: RSymbol, cycle: u32, tx_hash: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut claim_info = Self::claim_infos((&who, symbol, cycle, tx_hash)).ok_or(Error::<T>::HasNoClaimInfo)?;
			let act = Self::acts((symbol, cycle)).ok_or(Error::<T>::HasNoAct)?;
			let fund_addr = Self::fund_address().ok_or(Error::<T>::NoFundAddress)?;
			let now_block = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;

			let final_block = claim_info.mint_block.saturating_add(act.locked_blocks);

			let mut should_claim_amount = claim_info.total_reward.saturating_sub(claim_info.total_claimed);
			if now_block < final_block {
				let du_blocks = now_block.saturating_sub(claim_info.latest_claimed_block) as u128;
				should_claim_amount = multiply_by_rational(claim_info.total_reward, du_blocks, act.locked_blocks as u128).unwrap_or(u128::MIN) as u128;
			}
			ensure!(<T as staking::Trait>::Currency::free_balance(&fund_addr).saturated_into::<u128>()> should_claim_amount,Error::<T>::InsufficientFis);

			//update state
			T::Currency::transfer(&fund_addr, &who, should_claim_amount.saturated_into(), KeepAlive)?;
			claim_info.total_claimed = claim_info.total_claimed.saturating_add(should_claim_amount);
			claim_info.latest_claimed_block = now_block;

			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {
	/// update user claim info when user mint rtoken
	pub fn update_claim_info(
		who: &T::AccountId,
		symbol: RSymbol,
		mint_value: u128,
		tx_hash: Vec<u8>,
	) {
		let cycle = Self::act_current_cycle(symbol);
		if cycle == 0 {
			return;
		}
		let act_op = Self::acts((symbol, cycle));
		if act_op.is_none() {
			return;
		}
		let mut act = act_op.unwrap();
		let now_block = <system::Module<T>>::block_number().try_into().ok().unwrap() as BlockNumber;
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
		act.left_amount = act.total_reward.saturating_sub(should_reward_amount);

		let claim_info = ClaimInfo {
			total_reward: should_reward_amount,
			total_claimed: 0,
			latest_claimed_block: now_block,
			mint_block: now_block,
		};
		//update state
		<ClaimInfos<T>>::insert((who, symbol, cycle, tx_hash.clone()), claim_info);
		let mut acts = Self::user_acts((who, symbol)).unwrap_or(vec![]);
		if !acts.contains(&cycle) {
			acts.push(cycle);
			<UserActs<T>>::insert((who, symbol), acts);
		}
		let mut mints = Self::user_mints((who, symbol, cycle)).unwrap_or(vec![]);
		if !mints.contains(&tx_hash) {
			mints.push(tx_hash);
			<UserMints<T>>::insert((who, symbol, cycle), mints);
		}
	}

	/// update current act cycle when block finalize
	fn update_act_current_cycle(now: BlockNumber, symbol: RSymbol) {
		let cycle = Self::act_latest_cycle(symbol);
		if cycle > 0 {
			let last_current_cycle = Self::act_current_cycle(symbol);
			if cycle == last_current_cycle {
				return;
			}

			let mut begin = 1;
			if last_current_cycle > 0 {
				begin = last_current_cycle;
			}
			for i in begin..(cycle + 1) {
				let act_op = Self::acts((symbol, i));
				if act_op.is_none() {
					continue;
				}
				let act = act_op.unwrap();
				if act.begin <= now && act.end >= now {
					if i != last_current_cycle {
						<ActCurrentCycle>::insert(symbol, i);
					}
					break;
				}
			}
		}
	}
}
