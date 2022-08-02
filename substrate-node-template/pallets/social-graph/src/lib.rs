#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{
			StaticLookup
		}
	};


	/// Value an account attaches to their attestation representing their 
	/// confidence in the target account's validity
	type Confidence = u8;

	/// A count of attestations
	type AttestCount = u32;

	/// A sum of confidence values
	type ConfidenceSum = u32;

	/// Voter's decision on a challenge: -10..10 (inclusive) 
	/// Greater numbers indicate greater suspicion 
	type Vote = i8;


	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// consider adding MAX confidence value to change in runtime w votes
		
		#[pallet::constant]
		/// Number of blocks the each challenge stays active
		type ChallengeDuration: Get<u32>;

		// Mayber add minimum stake for challenge and vote 

		#[pallet::constant]
		/// Maximum number of challenges that can be active at once
		type MaxChallenges: Get<u32>;

	}


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


	// Attestation storage types:
	#[pallet::storage]
	#[pallet::getter(fn attestations)]
	/// Attestations is a double storage map holding
	pub type Attestations<T: Config> = StorageDoubleMap<_, Blake2_128Concat, 
		T::AccountId, Blake2_128Concat, T::AccountId, (Confidence, T::BlockNumber)>;

	#[pallet::storage]
	#[pallet::getter(fn account_data)]
	/// All accounts' data (# attestations, sum of confidence, birth block).
	pub type AccountData<T: Config> = CountedStorageMap<_, Blake2_128Concat, 
		T::AccountId, (AttestCount, ConfidenceSum, T::BlockNumber)>;

	#[pallet::storage]
	#[pallet::getter(fn attest_count)]
	/// All accounts' data (# attestations, sum of confidence, birth block).
	pub type AttestationCount<T: Config> = StorageValue<_, u32>;


	// Challenge Storage types:
	#[pallet::storage]
	#[pallet::getter(fn challenges)]
	/// A bounded vec containing all active challenges. Each challenge is 
	/// represented by a tuple containing the suspect's account ID and the 
	/// final block.
	pub type Challenges<T: Config> = StorageValue<_,
		BoundedVec<(T::AccountId, T::BlockNumber), T::MaxChallenges>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn votes)]
	/// A double storage map containing the suspect's ID as key1, the voter
	/// ID as key2, and the vote value 
	pub type Votes<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vote>;
	



	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Attested(T::AccountId, T::AccountId, (Confidence, T::BlockNumber)),
		/// A challence was issued! (Challenger, Suspect, FinalBlock)
		ChallengeCreated(T::AccountId, T::AccountId, T::BlockNumber),
	}


	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The chosen confidence value is not within valid bounds: 0..10 (inclusive).
		ConfidenceOutOfBounds, 
		/// An account may not attest to themselves.
		SelfAttestationError,
		/// The party trying to attest has not been attested for.
		UnknownAttester,
		/// Account must have >= network average attest count to attest
		InsufficientAttestCount,
		/// Account must have >= average confindence on network to attest
		InsufficientConfidence,

		/// Vote must be within -10..10 (inclusive)
		VoteOutOfBounds,
		/// The account does not meet network requirements to submit a challenge.
		InvalidChallenger,
		/// The maximum permissible number of challenges has been reached.
		MaxChallengesReached,
		
	}


	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		//Weight: see `begin_block`
		fn on_initialize(n: T::BlockNumber) -> Weight {
			Self::begin_block(n)
		}
	}



	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Takes in an origin account and a target account along with an 
		/// attestation. The origin attests for the target's personhood with a 
		/// confidence value 0..10 (inclusive). The confidence along with the 
		/// current block number are written to a double map `attestations`
		/// where the first key is the target being attested for and the second
		/// is the origin who is sending their attestation. The origin cannot
		/// attest for themselves.
		///
		/// Todo: Attestations only valid if the origin has >= the average #
		/// attestations for the network and >= the average confidence for the
		/// network
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn attest(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
			confidence: Confidence,
		) -> DispatchResult {
			// Ensure that confidence is within the valid range 0..10 (inclusive).
			ensure!(confidence <= 10, Error::<T>::ConfidenceOutOfBounds);
			// Check origin is signed and lookup the target.
			let origin = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(target)?;
			// Ensure that origin and dest are not the same account.
			ensure!(origin.clone() != dest.clone(), Error::<T>::SelfAttestationError);
			
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Get latest attest count, initialize if empty
			let total_attest_count = match <AttestationCount<T>>::try_get() {
				Ok(value) => value,
				Err(_) => {
					<AttestationCount<T>>::put(0);
					0
				}
			};


			// Deconstruct the origin's Account Data for validity checks.
			let (origin_attest_count, _origin_sum_confidence, _origin_birth_block) = 
				match <AccountData<T>>::try_get(origin.clone()) {
					Ok(data) => data,
					Err(_) => ({
						<AccountData<T>>::insert(origin.clone(), (0, 0, current_block)); // add attester to AccountData if they dont exist yet
						(0, 0, current_block) 
					}),		
				};

			// Ensure # attestations for origin >= the average for the network
			ensure!(
				origin_attest_count >= total_attest_count / <AccountData<T>>::count(), 
				Error::<T>::InsufficientAttestCount);

			// Todo: Ensure avg confidence >= the average of the network
			// Todo: Ensure the account has existed long enough
			


			// Update storage (Attestations and Account Data).
			
			if <Attestations<T>>::contains_key(dest.clone(), origin.clone()) { // if attestations contains the key pair already it means we're chanching values of an existing attestation

				let confidence_diff = confidence - <Attestations<T>>::get(dest.clone(), origin.clone()).unwrap().0; // calculate the difference between new and old confidence values to update AccountData

				let (og_count, og_confidence, birth_block) = <AccountData<T>>::get(dest.clone()).unwrap(); // deconstruct latest AccountData for reference

				// Update account data.
				<AccountData<T>>::insert(dest.clone(), (
					og_count, // do not increment because key pair already exists  
					og_confidence + confidence_diff as u32, // add the diff
					birth_block)); // leave birth block unchainged 

			} else { // if the key pair doesn't exist yet, this is a new attestation

				// Increment the total attest count.	
				<AttestationCount<T>>::put(total_attest_count + 1);

				if <AccountData<T>>::contains_key(dest.clone()) { // account 

					let (og_count, og_confidence, birth_block) = <AccountData<T>>::get(dest.clone()).unwrap(); // deconstruct latest AccountData for reference
						
					// Update account data.
					<AccountData<T>>::insert(dest.clone(), (
						og_count + 1, // key pair did not exist so add new attestation to the original count 
						og_confidence + confidence as u32, // add confidence for new attestation to the original sum
						birth_block)); // leave birth block because the account is not new

				} else { // if the account is new initialize all AccountData
					<AccountData<T>>::insert(dest.clone(), (1, confidence as u32, current_block))
				}
			}
			// Update Attestations.
			<Attestations<T>>::insert(dest.clone(), origin.clone(), (confidence, current_block));
			
			
			// Emit an event.
			Self::deposit_event(Event::Attested(origin, dest, (confidence, current_block)));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}


		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		/// Begin a challenge on another account's validity. Every account will
		/// get opportunity to cast a vote to decide whether to keep or ban the
		/// suspect 
		pub fn challenge(
			challenger: OriginFor<T>,
			suspect: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {

			let challenger = ensure_signed(challenger)?;
			let suspect = T::Lookup::lookup(suspect)?;

			// Check challenger validity
			ensure!(Self::check_account_validity(challenger.clone()), Error::<T>::InvalidChallenger);

			// Calculate final block
			let current_block = <frame_system::Pallet<T>>::block_number();
			let final_block = current_block + T::ChallengeDuration::get().into();

			// Add challenge to challenges
			let mut challenges = <Challenges<T>>::get();
			match challenges.try_insert(0,(suspect.clone(), final_block)) {
				Ok(_) => (),
				Err(_) => return Err(Error::<T>::MaxChallengesReached.into())
			};
			<Challenges<T>>::put(challenges);

			// Emit an event.
			Self::deposit_event(Event::ChallengeCreated(challenger, suspect, final_block));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}


		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		/// Vote
		pub fn vote(origin: OriginFor<T>, suspect: T::AccountId, value: Vote) -> DispatchResult{
			todo!();
			let origin = ensure_signed(origin)?;
			// Check value validity.
			// Check voter validity.
			// Add vote to storage.

			// Emit an event.
		}
	}

	// Helper functions.
	impl<T: Config> Pallet<T> {

		/// Initializes a block by processing and removing completed challenges. 
		fn begin_block(block_number: T::BlockNumber) -> Weight {
			let current_block = <frame_system::Pallet<T>>::block_number();
			let mut challenges = <Challenges<T>>::get();

			let mut i: usize = 0;
			let (mut suspect, mut block) = match challenges.pop() {
				Some(tup) => tup,
				None => return T::BlockWeights::get().base_block,
			};
			while block >= current_block {
				// Tally votes
				// Enact final judgement

				// Prepare for next round
				(suspect, block) = match challenges.pop() {
					Some(tup) => tup,
					None => return T::BlockWeights::get().base_block,
				};
				i += 1;
			}
			match challenges.try_push((suspect, block)) {
				Ok(_) => (),
				Err(_) => (), // This should theoretically never happen because we would have just removed a tuple 
			};
			<Challenges<T>>::put(challenges);

			T::BlockWeights::get().base_block
		}

		/// Checks whether Account is eligible to attest/vote/challenge
		fn check_account_validity(account: T::AccountId) -> bool {
			todo!();
			// Avg confidence is above network average
			// # attestations is above network average
			// Account is not banned from the network 
		}


		/// Tallies the votes from a challenge
		fn tally(suspect: T::AccountId) -> u32{
			todo!();
		}
	}
}