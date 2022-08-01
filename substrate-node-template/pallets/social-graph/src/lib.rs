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


	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn attestations)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	// Attestations is a double storage map holding
	pub type Attestations<T: Config> = StorageDoubleMap<_, Blake2_128Concat, 
		T::AccountId, Blake2_128Concat, T::AccountId, (u8, T::BlockNumber)>;


	#[pallet::storage]
	#[pallet::getter(fn account_data)]
	// All accounts' data (# attestations, sum of confidence, birth block).
	pub type AccountData<T: Config> = CountedStorageMap<_, Blake2_128Concat, 
		T::AccountId, (u32, u32, T::BlockNumber)>;


	#[pallet::storage]
	#[pallet::getter(fn attest_count)]
	// All accounts' data (# attestations, sum of confidence, birth block).
	pub type AttestationCount<T: Config> = StorageValue<_, u32>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Attested(T::AccountId, T::AccountId, (u8, T::BlockNumber)),
		TotalCount(u32),
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
	}


	//#[pallet::hooks]
	//impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		//fn on_initialize(n: T::BlockNumber) -> Weight {
		//}
	//}



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
		/// Todo: 
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn attest(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
			confidence: u8,
		) -> DispatchResult {
			// Ensure that confidence is within the valid range 0..10 (inclusive).
			ensure!(confidence <= 10, Error::<T>::ConfidenceOutOfBounds);
			// Check origin is signed and lookup the target.
			let origin = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(target)?;
			// Ensure that origin and dest are not the same account.
			ensure!(origin.clone() != dest.clone(), Error::<T>::SelfAttestationError);
			
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Find get latest attest count, initialize if empty
			let latest_attest_count = match <AttestationCount<T>>::try_get() {
				Ok(value) => value,
				Err(_) => {
					<AttestationCount<T>>::put(0);
					0
				}
			};


			// Deconstruct the origin's Account Data for validity checks.
			let (origin_attest_count, origin_sum_confidence, origin_birth_block) = 
				match <AccountData<T>>::try_get(origin.clone()) {
					Ok(data) => data,
					Err(_) => return Err(Error::<T>::UnknownAttester.into()),
				};

			// Ensure # attestations for origin >= the average for the network
			
			ensure!(
				origin_attest_count >= latest_attest_count / <AccountData<T>>::count(), 
				Error::<T>::InsufficientAttestCount);

			// Ensure avg confidence >= the average of the network

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
				<AttestationCount<T>>::put(latest_attest_count + 1);

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
	}

	// Helper functions.
	impl<T: Config> Pallet<T> {

		// Counts the number of attestations for a given account.

		// Sums the confidence for a given account

		// Collects a list of attesters for a given account

		// Calculates the average confidence of every account on the network

		// Finds the earliest attestation for a given account

		// Counts all accounts attested for in double map

	}
}