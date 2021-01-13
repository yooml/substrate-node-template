#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, StorageMap, traits::Randomness, Parameter
};
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;
use sp_runtime::{DispatchError,traits::{AtLeast32Bit,Bounded}};


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Decode, Encode)]
pub struct Kitty(pub [u8; 16]);

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
	type KittyIndex: Parameter + AtLeast32Bit + Bounded + Default + Copy;
}

decl_storage! {
    trait Store for Module<T: Trait> as Kitties {
        Something get(fn something): Option<u32>;
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
        pub OwnedKitties get(fn owned_kitties): map hasher(blake2_128_concat) T::AccountId => Option<T::KittyIndex>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        KittyIndex = <T as Trait>::KittyIndex,
    {
        Created(AccountId, KittyIndex),
        Transferred(AccountId, AccountId, KittyIndex),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        KittiesCountOverflow,
        InvalidKittyId,
        RequireDifferentParent,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[weight = 0]
        pub fn create(origin, something: u32) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);
            Self::insert_kitty(&sender,kitty_id,kitty);
            // Self::deposit_event(RawEvent::Created(sender, kitty_id))
            Ok(Self::deposit_event(RawEvent::Created(sender, kitty_id)))
        }

        #[weight = 0]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            <KittyOwners<T>>::insert(kitty_id, to.clone());
            OwnedKitties::<T>::remove(&sender);
            OwnedKitties::<T>::insert(&to, kitty_id);
            Self::deposit_event(RawEvent::Transferred(sender, to , kitty_id));
        }

        #[weight = 0]
        pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        }
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        <Kitties::<T>>::insert(kitty_id, kitty);
        <KittiesCount::<T>>::put(kitty_id + 1.into());
        <KittyOwners<T>>::insert(kitty_id, owner);
        <OwnedKitties<T>>::insert(owner, kitty_id);
    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count();
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = {
            T::Randomness::random_seed();
            &sender;
            <frame_system::Module<T>>::extrinsic_index();
        };
        payload.using_encoded(blake2_128)
    }

    fn do_breed(
        sender: &T::AccountId,
        kitty_id_1: T::KittyIndex,
        kitty_id_2: T::KittyIndex,
    ) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);
        let kitty_id = Self::next_kitty_id()?;

        let kitty1_dna = kitty1.0;
        let kitty2_dna = kitty2.0;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }
        Self::insert_kitty(sender, kitty_id, Kitty(new_dna));
        Ok(kitty_id)
    }
}
