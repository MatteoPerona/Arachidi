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
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	// Attestations is a double storage map holding
	pub type Attestations<T: Config> = StorageDoubleMap<_, Blake2_128Concat, 
		T::AccountId, Blake2_128Concat, T::AccountId, (u8, T::BlockNumber)>;

	// Todo: add a counted storage map with accountid as key and (count of 
	// attestations, sum confidence) as values

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Attested(T::AccountId, T::AccountId, (u8, T::BlockNumber)),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ConfidenceOutOfBounds, 
		SelfAttestationError,

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
			// Clone for event.
			let o = origin.clone();
			let d = dest.clone();

			let current_block = <frame_system::Pallet<T>>::block_number();

			// Update storage.
			<Attestations<T>>::insert(dest, origin, (confidence, current_block));

			// Emit an event.
			Self::deposit_event(Event::Attested(o, d, (confidence, current_block)));
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