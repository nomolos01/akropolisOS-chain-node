/// runtime module implementing the ERC20 token interface

use rstd::prelude::*;
use parity_codec::Codec;
use support::{dispatch::Result, StorageMap, Parameter, StorageValue, decl_storage, decl_module, decl_event, ensure};
use system::{self, ensure_signed};
use runtime_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic, As};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Token {
        Init get(is_init): bool;

        TotalSupply get(total_supply) config(): T::TokenBalance;
        BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
        Allowance get(allowance): map (T::AccountId, T::AccountId) => T::TokenBalance;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn transfer(origin, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            Self::_transfer(sender, to, value)
        } 

        pub fn approve(origin, spender: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(<BalanceOf<T>>::exists(&sender), "Account does not own this token");

            let allowance = Self::allowance((sender.clone(), spender.clone()));
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;

            <Allowance<T>>::insert((sender.clone(), spender.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(sender, spender, value));
            Ok(())
        }

        pub fn burn(origin, #[compact] value: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;

            Self::deposit_event(RawEvent::Burn(sender, value));

            Ok(())
        }

        // if approved, transfer from an account to another account without needing owner's signature
        pub fn transfer_from(_origin, from: T::AccountId, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
            ensure!(<Allowance<T>>::exists((from.clone(), to.clone())), "Allowance does not exist.");
            let allowance = Self::allowance((from.clone(), to.clone()));
            ensure!(allowance >= value, "Not enough allowance.");

            let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;

            <Allowance<T>>::insert((from.clone(), to.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(from.clone(), to.clone(), value));
            Self::_transfer(from, to, value)
        }
    }
}

// events
decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, TokenBalance = <T as self::Trait>::TokenBalance {
        Transfer(AccountId, AccountId, TokenBalance),
        Approval(AccountId, AccountId, TokenBalance),
        Burn(AccountId, TokenBalance),
    }
);

impl<T: Trait> Module<T> {
    pub fn init(owner: T::AccountId, ) -> Result {
        ensure!(Self::is_init() == false, "Token already initialized.");

        <BalanceOf<T>>::insert(owner, Self::total_supply());
        <Init<T>>::put(true);
        Ok(())
    }

    // internal transfer function for ERC20 interface
    fn _transfer(
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        ensure!(<BalanceOf<T>>::exists(from.clone()), "Account does not own this token");
        let sender_balance = Self::balance_of(from.clone());
        ensure!(sender_balance >= value, "Not enough balance.");
        let updated_from_balance = sender_balance.checked_sub(&value).ok_or("underflow in calculating balance")?;
        let receiver_balance = Self::balance_of(to.clone());
        let updated_to_balance = receiver_balance.checked_add(&value).ok_or("overflow in calculating balance")?;

        <BalanceOf<T>>::insert(from.clone(), updated_from_balance);
        <BalanceOf<T>>::insert(to.clone(), updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(from, to, value));
        Ok(())
    }
}


/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_noop, assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type TransactionPayment = ();
        type TransferPayment = ();
        type DustRemoval = ();
        type Event = ();
    }
    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
    }
    impl Trait for Test {
        type Event = ();
    }
    type TokenModule = Module<Test>;

    const TOKEN_NAME: &[u8; 10] = b"NAME";
    const TOKEN_DESC: &[u8; 10] = b"Description-1234_";
    const USER1: u64 = 1;
    const USER2: u64 = 2;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn create_token_init() {
        with_externalities(&mut new_test_ext(), || {
            // assert_eq!();
        })
    }
}