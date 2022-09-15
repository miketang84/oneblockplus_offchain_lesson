#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_system::{
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction,
        Signer,
    },
};
use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");
pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for OcwAuthId
        {
            type RuntimeAppPublic = Public;
            type GenericSignature = sp_core::sr25519::Signature;
            type GenericPublic = sp_core::sr25519::Public;
        }
}


#[frame_support::pallet]
pub mod pallet {
    use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    use sp_std::vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

        #[pallet::weight(0)]
        pub fn submit_data(origin: OriginFor<T>, payload: Vec<u8>) -> DispatchResultWithPostInfo {

            let _who = ensure_signed(origin)?;

            log::info!("in submit_data call: {:?}", payload);

            Ok(().into())
        }
	}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

        fn offchain_worker(block_number: T::BlockNumber) {
            log::info!("Hello World from offchain workers!: {:?}", block_number);

            let payload: Vec<u8> = vec![1,2,3,4,5,6,7,8];
            _ = Self::send_signed_tx(payload);

            log::info!("Leave from offchain workers!: {:?}", block_number);

        }

        fn on_initialize(_n: T::BlockNumber) -> Weight {
            log::info!("in on_initialize!");
            0
        }

        fn on_finalize(_n: T::BlockNumber) {
            log::info!("in on_finalize!");
        }

        fn on_idle(_n: T::BlockNumber, _remaining_weight: Weight) -> Weight {
            log::info!("in on_idle!");
            0
        }

    }

    impl<T: Config> Pallet<T> {

        fn send_signed_tx(payload: Vec<u8>) -> Result<(), &'static str> {
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
                return Err(
                    "No local accounts available. Consider adding one via `author_insertKey` RPC.",
                    )
            }

            let results = signer.send_signed_transaction(|_account| {

                Call::submit_data { payload: payload.clone() }
            });

            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted data:{:?}", acc.id, payload),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
            }

            Ok(())
        }
    }

}
