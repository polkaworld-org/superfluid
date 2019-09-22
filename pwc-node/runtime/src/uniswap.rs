/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, Dispatchable, Parameter,
    StorageDoubleMap, StorageMap, StorageValue,
};
use system::{ensure_root, ensure_signed};
//use parity_codec::{Encode, Decode};
use core::convert::TryInto;
use primitives::crypto::{UncheckedFrom, UncheckedInto, Wraps};
use rstd::prelude::*;
use rstd::{marker::PhantomData, mem, vec::Vec};
use sr_primitives::traits::{
    Bounded, Hash, Member, One, SaturatedConversion, SimpleArithmetic, StaticLookup, Zero,
};

/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The units in which we record balances.
    type Balance: Member + Parameter + SimpleArithmetic + Default + Copy;
    /// The arithmetic type of asset identifier.
    type AssetId: Parameter + SimpleArithmetic + Default + Copy;
    /// The exchange address type to make a new paired pool address as AccountId
    type ExchangeAddress: ExchangeFactory<Self::AssetId, Self::AccountId>;
    /// The global fee rate
    type FeeRate: Parameter + SimpleArithmetic + Default + Copy;
}

decl_event!(
    pub enum Event<T>
    where <T as system::Trait>::AccountId,
    <T as Trait>::Balance,
    <T as Trait>::AssetId {
        /// Some assets were issued.
        Issued(AssetId, AccountId, Balance),
        /// Some assets were transferred.
        Transferred(AssetId, AccountId, AccountId, Balance),
        /// Some assets were destroyed.
        Destroyed(AssetId, AccountId, Balance),
        /// Assets swapment event
        AssetsSwapped(AccountId, AssetId, Balance, AssetId, Balance),
        /// Adding liquidity event
        /// account, liquidity amount, paired asset_id
        LiquidityAdded(AccountId, Balance, AssetId),
        /// Removing liquidity event
        /// account, liquidity amount, paired asset_id
        LiquidityRemoved(AccountId, Balance, AssetId),
        /// The balance of an asset has changed
        ReserveChanged(AssetId, Balance),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Superfluid
    where
        u64: core::convert::From<<T as Trait>::AssetId>,
        u128: core::convert::From<<T as Trait>::Balance>,
    <T as Trait>::Balance: core::convert::From<u128>
    {
        /// The next asset identifier up for grabs.
        NextAssetId get(next_asset_id): T::AssetId;
        /// The total unit supply of an asset.
        TotalSupply get(get_asset_total_supply): map T::AssetId => T::Balance;
        /// The number of units of assets held by any given account.
        Balances get(get_asset_balance): map (T::AssetId, T::AccountId) => T::Balance;
        /// The default inherent asset in this platform
        InherentAsset get(inherent_asset_id): T::AssetId;
        /// The global fee rate of this platform
        FeeRateGlobal get(fee_rate) config(): T::FeeRate;
        /// Total liquidity of each pair pool (InherentAsset and another asset)
        TotalLiquidities get(total_liquidity): map T::AssetId => T::Balance;
        /// The liquidity of each account on some one asset pool
        AccountLiquidities get(account_liquidity): map (T::AssetId, T::AccountId) => T::Balance;
        /// for test only
        Owner get(owner) config(): T::AccountId;
        /// Accounts of exchanges
        ExchangeAccounts get(exchange_account): map T::AssetId => T::AccountId
    }

    add_extra_genesis {
        config(assets): Vec<(T::AccountId, T::Balance)>;

        build(|config: &GenesisConfig<T>| {
            for asset in config.assets.iter() {
                let (account, amount) = asset;
                <Module<T>>::_issue(account.clone(), amount.clone());
                let to_account = <Owner<T>>::get();
                let asset_id = <NextAssetId<T>>::get() - 1.into();
                <Module<T>>::transfer(account.clone(), asset_id, to_account, 50000.into());
            }
        })
    }

}

// The module's dispatchable functions.
decl_module! {
    pub struct Module<T: Trait> for enum Call where
        origin: T::Origin,
        u64: core::convert::From<<T as Trait>::AssetId>,
        u128: core::convert::From<<T as Trait>::Balance>,
    <T as Trait>::Balance: core::convert::From<u128>
    {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        /// Issue a new class of fungible assets. There are, and will only ever be, `total`
        /// such assets and they'll all belong to the `origin` initially. It will have an
        /// identifier `AssetId` instance: this will be specified in the `Issued` event.
        /// This will make a increased id asset.
        /// @origin
        /// @total    How much balance of new asset
        fn issue(origin, total: T::Balance) -> Result {
            let origin = ensure_signed(origin)?;

            let id = Self::next_asset_id();
            <NextAssetId<T>>::mutate(|id| *id += One::one());

            <Balances<T>>::insert((id, origin.clone()), total);
            <TotalSupply<T>>::insert(id, total);

            // debug
            runtime_io::print("----> asset id, total balance");
            let idn: u64 = id.into();
            runtime_io::print(idn);
            let b: u128 = <Balances<T>>::get((id, origin.clone())).into();
            runtime_io::print(b as u64);

            Self::deposit_event(RawEvent::Issued(id, origin, total));

            Ok(())
        }

        /// Destroy any assets of `id` owned by `origin`.
        /// @origin
        /// @id      Asset id to be destroyed
        fn destroy(origin, id: T::AssetId) -> Result {
            let origin = ensure_signed(origin)?;
            let balance = <Balances<T>>::take((id, origin.clone()));
            ensure!(!balance.is_zero(), "origin balance should be non-zero");

            <TotalSupply<T>>::mutate(id, |total_supply| *total_supply -= balance);
            Self::deposit_event(RawEvent::Destroyed(id, origin, balance));

            Ok(())
        }

        /// Set the default inherent asset
        /// @origin
        /// @asset    The asset to become inherent asset
        pub fn set_inherent_asset(origin, asset: T::AssetId) -> Result {
            //ensure_root(origin)?;
            <InherentAsset<T>>::mutate(|ia| *ia = asset.clone());

            // debug
            runtime_io::print("----> Inhere Asset Id");
            let b: u64 = Self::inherent_asset_id().into();
            runtime_io::print(b);

            Ok(())
        }

        /// Set global fee rate, need root permission
        /// @origin
        /// @fee_rate    the global fee rate on each transaction
        pub fn set_fee_rate(origin, fee_rate: T::FeeRate) -> Result {
            //ensure_root(origin)?;
            <FeeRateGlobal<T>>::mutate(|fr| *fr = fee_rate);

            Ok(())
        }

        /// Transfer an asset to another account
        pub fn transfer_asset(origin,
                    id: T::AssetId,
                    to_account: T::AccountId,
                    amount: T::Balance
        ) -> Result {
            let from_account = ensure_signed(origin)?;
            Self::transfer(from_account, id, to_account, amount);

            Ok(())
        }

        /// Swap two assets, input amount is exact
        /// @origin
        /// @output_account    The recipient of output asset
        /// @asset_input       Input asset id
        /// @asset_output      Output asset id
        /// @input_amount      The exact input amount of input asset
        pub fn swap_assets_with_exact_input(
            origin,
            output_account: T::AccountId,
            asset_input: T::AssetId,
            asset_output: T::AssetId,
            input_amount: T::Balance,
            min_output: T::Balance) -> Result {

            let input_account = ensure_signed(origin)?;
            let inherent_asset_id = Self::inherent_asset_id();
            // check


            let fee_rate = Self::fee_rate();
            if asset_input == inherent_asset_id {
                // inherent asset to another asset
                Self::inherent_asset_to_paired_asset_with_exact_input(
                    input_account,
                    output_account,
                    asset_output,
                    input_amount,
                    min_output,
                    fee_rate
                )?;
            }
            else if asset_output == inherent_asset_id {
                // another asset to inherent asset
                Self::paired_asset_to_inherent_asset_with_exact_input(
                    input_account,
                    output_account,
                    asset_input,
                    input_amount,
                    min_output,
                    fee_rate
                )?;
            }
            else {
                // asset A to asset B
                Self::asset_a_to_asset_b_with_exact_input(
                    input_account,
                    output_account,
                    asset_input,
                    asset_output,
                    input_amount,
                    min_output,
                    fee_rate
                )?;

            }

            Ok(())
        }

        /// Swap two assets, output is exact
        /// @origin
        /// @output_account    The recipient of output asset
        /// @asset_input       Input asset id
        /// @asset_output      Output asset id
        /// @output_amount     The exact output amount of output asset
        /// @max_input         The limitation of max amount input asset
        pub fn swap_assets_with_exact_output(
            origin,
            output_account: T::AccountId,
            asset_input: T::AssetId,
            asset_output: T::AssetId,
            output_amount: T::Balance,
            max_input: T::Balance) -> Result {

            let input_account = ensure_signed(origin)?;
            let inherent_asset_id = Self::inherent_asset_id();
            // check


            let fee_rate = Self::fee_rate();
            if asset_input == inherent_asset_id {
                // inherent asset to another asset
                Self::inherent_asset_to_paired_asset_with_exact_output(
                    input_account,
                    output_account,
                    asset_output,
                    output_amount,
                    max_input,
                    fee_rate
                )?;
            }
            else if asset_output == inherent_asset_id {
                // another asset to inherent asset
                Self::paired_asset_to_inherent_asset_with_exact_output(
                    input_account,
                    output_account,
                    asset_input,
                    output_amount,
                    max_input,
                    fee_rate
                )?;
            }
            else {
                // asset A to asset B
                Self::asset_a_to_asset_b_with_exact_output(
                    input_account,
                    output_account,
                    asset_input,
                    asset_output,
                    output_amount,
                    max_input,
                    fee_rate
                )?;

            }

            Ok(())
        }

        /// Add liquidity to a pool
        /// @origin
        /// @asset_id    The (inherent_asset_id, asset_id) pair pool to inject liquidity
        /// @inherent_asset_amount    The exact amount of inherent asset to be injected
        /// @asset_amount             The amount of paired asset to be injected
        /// @min_liquidity            The minimum liquidity required to be injected once
        pub fn add_liquidity(
            origin,
            asset_id: T::AssetId,
            inherent_asset_amount: T::Balance,
            asset_amount: T::Balance,
            min_liquidity: T::Balance) -> Result {
            let account = ensure_signed(origin)?;

            Self::_add_liquidity(
                account.clone(),
                asset_id,
                inherent_asset_amount,
                asset_amount,
                min_liquidity
            );

            Self::deposit_event(RawEvent::LiquidityAdded(account, inherent_asset_amount, asset_id));

            Ok(())
        }

        /// Remove liquidity from a pool
        /// @origin
        /// @asset_id    The (inherent_asset_id, asset_id) pair pool to inject liquidity
        /// @liquidity   The exact amount liquidity to be removed
        /// @min_inherent_asset_amount    The minimum amount of inherent asset to be removed
        /// @min_asset_amount             The minimum amount of paired asset to be removed
        fn remove_liquidity(
            origin,
            asset_id: T::AssetId,
            liquidity: T::Balance,
            min_inherent_asset_amount: T::Balance,
            min_asset_amount: T::Balance) -> Result {
            let account = ensure_signed(origin)?;

            Self::_remove_liquidity(
                account.clone(),
                asset_id,
                liquidity,
                min_inherent_asset_amount,
                min_asset_amount,
            );

            Self::deposit_event(RawEvent::LiquidityRemoved(account, liquidity, asset_id));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T>
where
    u64: core::convert::From<<T as Trait>::AssetId>,
    u128: core::convert::From<<T as Trait>::Balance>,
    <T as Trait>::Balance: core::convert::From<u128>,
{
    /// Issue a new class of fungible assets. There are, and will only ever be, `total`
    /// such assets and they'll all belong to the `origin` initially. It will have an
    /// identifier `AssetId` instance: this will be specified in the `Issued` event.
    /// This will make a increased id asset.
    /// @origin
    /// @total    How much balance of new asset
    fn _issue(account: T::AccountId, total: T::Balance) -> rstd::result::Result<(), &'static str> {
        let id = Self::next_asset_id();
        <NextAssetId<T>>::mutate(|id| *id += One::one());

        <Balances<T>>::insert((id, account.clone()), total);
        <TotalSupply<T>>::insert(id, total);

        // debug
        runtime_io::print("----> asset id, total balance");
        let idn: u64 = id.into();
        runtime_io::print(idn);
        let b: u128 = <Balances<T>>::get((id, account.clone())).into();
        runtime_io::print(b as u64);

        Self::deposit_event(RawEvent::Issued(id, account, total));

        Ok(())
    }

    /// Move some assets from one holder to another.
    /// @from_account    The account lost amount of a certain asset balance
    /// @id              The asset id to be transfered
    /// @to_account      The account receive the sent asset balance
    /// @amount          The amount value to be transfered
    fn transfer(
        from_account: T::AccountId,
        id: T::AssetId,
        to_account: T::AccountId,
        amount: T::Balance,
    ) -> rstd::result::Result<(), &'static str> {
        let origin_account = (id, from_account.clone());
        let origin_balance = <Balances<T>>::get(&origin_account);
        let target = to_account;
        ensure!(!amount.is_zero(), "transfer amount should be non-zero");
        ensure!(
            origin_balance >= amount,
            "origin account balance must be greater than or equal to the transfer amount"
        );

        Self::deposit_event(RawEvent::Transferred(
            id,
            from_account,
            target.clone(),
            amount,
        ));
        <Balances<T>>::insert(origin_account, origin_balance - amount);
        <Balances<T>>::mutate((id, target), |balance| *balance += amount);

        Ok(())
    }

    /// Input inherent asset, output paired asset, with exact input amount
    /// @input_account    The account to send inherent asset to paired pool
    /// @output_account   The account to receive paired asset from paired pool
    /// @paired_asset_id  The paired asset, used to represent which paired pool to act
    /// @input_amount     The amount of input inherent asset to paired pool
    /// @min_output_amount    The limitation setting of minimum amount output paired asset
    /// @fee_rate         The fee rate used to calculate the handing charge
    fn inherent_asset_to_paired_asset_with_exact_input(
        input_account: T::AccountId,
        output_account: T::AccountId,
        paired_asset_id: T::AssetId,
        input_amount: T::Balance,
        min_output_amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        let paired_asset_output_amount =
            Self::calc_paired_asset_output_amount(paired_asset_id, input_amount, fee_rate)?;

        // check paired_asset_output_amount > 0
        // check paired_asset_output_amount >= min_output_amount
        let inherent_asset_id = Self::inherent_asset_id();

        // check input_account's balance > input_amount

        let exchange_address = Self::get_exchange_address(inherent_asset_id, paired_asset_id);

        // do transfer
        Self::transfer(
            input_account.clone(),
            inherent_asset_id.clone(),
            exchange_address.clone(),
            input_amount,
        );
        Self::transfer(
            exchange_address.clone(),
            paired_asset_id.clone(),
            output_account,
            paired_asset_output_amount,
        );

        // debug
        runtime_io::print("----> exchange inherent asset balance, exchange paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(paired_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print("----> account inherent asset balance, account paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), input_account.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(paired_asset_id.clone(), input_account.clone())).into();
        runtime_io::print(paired_b as u64);

        Self::deposit_event(RawEvent::AssetsSwapped(
            input_account,
            inherent_asset_id,
            input_amount,
            paired_asset_id,
            paired_asset_output_amount,
        ));

        // emit event
        let asset_balance_in_pool = Self::balance(paired_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            paired_asset_id,
            asset_balance_in_pool,
        ));
        let inherent_asset_balance_in_pool =
            Self::balance(inherent_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_pool,
        ));

        Ok(paired_asset_output_amount)
    }

    /// Input inherent asset, ouput paired asset at given fee_rate
    /// and exact output paired asset
    /// @input_account    The account to send inherent asset to paired pool
    /// @output_account   The account to receive paired asset from paired pool
    /// @paired_asset_id  The paired asset, used to represent which paired pool to act
    /// @output_amount    The amount of output paired asset from paired pool
    /// @max_input_amount    The limitation setting of maximum amount input inherent asset
    /// @fee_rate         The fee rate used to calculate the handing charge
    fn inherent_asset_to_paired_asset_with_exact_output(
        input_account: T::AccountId,
        output_account: T::AccountId,
        paired_asset_id: T::AssetId,
        output_amount: T::Balance,
        max_input_amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        let inherent_asset_input_amount =
            Self::calc_inherent_asset_input_amount(paired_asset_id, output_amount, fee_rate)?;

        // check inherent_asset_input_amount > 0
        // check inherent_asset_input_amount <= max_input_amount
        // check input_account's balance >= inherent_asset_input_amount

        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address =
            Self::get_exchange_address(inherent_asset_id.clone(), paired_asset_id);

        // do transfer
        Self::transfer(
            input_account.clone(),
            inherent_asset_id,
            exchange_address.clone(),
            inherent_asset_input_amount,
        );
        Self::transfer(
            exchange_address.clone(),
            paired_asset_id.clone(),
            output_account,
            output_amount,
        );

        // debug
        runtime_io::print("----> exchange inherent asset balance, exchange paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(paired_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print("----> account inherent asset balance, account paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), input_account.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(paired_asset_id.clone(), input_account.clone())).into();
        runtime_io::print(paired_b as u64);

        Self::deposit_event(RawEvent::AssetsSwapped(
            input_account,
            inherent_asset_id,
            inherent_asset_input_amount,
            paired_asset_id,
            output_amount,
        ));

        // emit event
        let asset_balance_in_pool = Self::balance(paired_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            paired_asset_id,
            asset_balance_in_pool,
        ));
        let inherent_asset_balance_in_pool =
            Self::balance(inherent_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_pool,
        ));

        Ok(inherent_asset_input_amount)
    }

    /// Input paired asset, output inherent asset
    /// Give exact paired asset input amount, expect min inherent asset output amount
    /// @input_account    The account to send paired asset to paired pool
    /// @output_account   The account to receive inherent asset from paired pool
    /// @paired_asset_id  The paired asset, used to represent which paired pool to act
    /// @input_amount     The amount of input paired asset to paired pool
    /// @min_output_amount    The limitation setting of minimum amount output inherent asset
    /// @fee_rate         The fee rate used to calculate the handing charge
    fn paired_asset_to_inherent_asset_with_exact_input(
        input_account: T::AccountId,
        output_account: T::AccountId,
        paired_asset_id: T::AssetId,
        input_amount: T::Balance,
        min_output_amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        let inherent_asset_output_amount =
            Self::calc_inherent_asset_output_amount(paired_asset_id, input_amount, fee_rate)?;

        // check input_account's balance > input_amount
        // check inherent_asset_output_amount > 0
        // check inherent_asset_output_amount >= min_output_amount

        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address =
            Self::get_exchange_address(inherent_asset_id.clone(), paired_asset_id);

        // check inherent asset balance of exchange poll >= inherent_asset_output_amount

        // do transfer
        Self::transfer(
            input_account.clone(),
            paired_asset_id.clone(),
            exchange_address.clone(),
            input_amount,
        );
        Self::transfer(
            exchange_address.clone(),
            inherent_asset_id.clone(),
            output_account,
            inherent_asset_output_amount,
        );

        // debug
        runtime_io::print("----> exchange inherent asset balance, exchange paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(paired_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print("----> account inherent asset balance, account paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), input_account.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(paired_asset_id.clone(), input_account.clone())).into();
        runtime_io::print(paired_b as u64);

        Self::deposit_event(RawEvent::AssetsSwapped(
            input_account,
            paired_asset_id,
            input_amount,
            inherent_asset_id,
            inherent_asset_output_amount,
        ));

        // emit event
        let asset_balance_in_pool = Self::balance(paired_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            paired_asset_id,
            asset_balance_in_pool,
        ));
        let inherent_asset_balance_in_pool =
            Self::balance(inherent_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_pool,
        ));

        Ok(inherent_asset_output_amount)
    }

    /// Input paired asset, output inherent asset
    /// Give exact inherent asset output amount, supply max paired asset input amount
    /// @input_account    The account to send paired asset to paired pool
    /// @output_account   The account to receive inherent asset from paired pool
    /// @paired_asset_id  The paired asset, used to represent which paired pool to act
    /// @output_amount    The amount of output paired asset from paired pool
    /// @max_input_amount    The limitation setting of maximum amount input paired asset
    /// @fee_rate         The fee rate used to calculate the handing charge
    fn paired_asset_to_inherent_asset_with_exact_output(
        input_account: T::AccountId,
        output_account: T::AccountId,
        paired_asset_id: T::AssetId,
        output_amount: T::Balance,
        max_input_amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        let paired_asset_input_amount =
            Self::calc_paired_asset_input_amount(paired_asset_id, output_amount, fee_rate)?;

        // check input_account's balance > input_amount
        // check inherent_asset_output_amount > 0
        // check inherent_asset_output_amount >= min_output_amount

        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address =
            Self::get_exchange_address(inherent_asset_id.clone(), paired_asset_id);

        // check inherent asset balance of exchange poll >= inherent_asset_output_amount

        // do transfer
        Self::transfer(
            input_account.clone(),
            paired_asset_id,
            exchange_address.clone(),
            paired_asset_input_amount,
        );
        Self::transfer(
            exchange_address.clone(),
            inherent_asset_id.clone(),
            output_account,
            output_amount,
        );

        // debug
        runtime_io::print("----> exchange inherent asset balance, exchange paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(paired_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print("----> account inherent asset balance, account paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), input_account.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(paired_asset_id.clone(), input_account.clone())).into();
        runtime_io::print(paired_b as u64);

        Self::deposit_event(RawEvent::AssetsSwapped(
            input_account,
            paired_asset_id,
            paired_asset_input_amount,
            inherent_asset_id,
            output_amount,
        ));

        // emit event
        let asset_balance_in_pool = Self::balance(paired_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            paired_asset_id,
            asset_balance_in_pool,
        ));
        let inherent_asset_balance_in_pool =
            Self::balance(inherent_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_pool,
        ));

        Ok(paired_asset_input_amount)
    }

    /// Asset a to asset b, with exact input amount
    /// @input_account    The account to send asset a
    /// @output_account   The account to receive asset b
    /// @asset_a          The id of asset a
    /// @asset_b          The id of asset b
    /// @input_amount     The amount of input asset a
    /// @min_output_amount    The minimum amount of ouput asset b
    /// @fee_rate         The fee rate used to calculate the handing charge
    fn asset_a_to_asset_b_with_exact_input(
        input_account: T::AccountId,
        output_account: T::AccountId,
        asset_a: T::AssetId,
        asset_b: T::AssetId,
        input_amount: T::Balance,
        min_output_amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_a_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_a);
        let exchange_b_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_b);

        let inherent_asset_output_amount =
            Self::calc_inherent_asset_output_amount(asset_a, input_amount, fee_rate)?;
        let asset_b_output_amount =
            Self::calc_paired_asset_output_amount(asset_b, inherent_asset_output_amount, fee_rate)?;

        // CHECKS

        // do transfer
        Self::transfer(
            input_account.clone(),
            asset_a.clone(),
            exchange_a_address.clone(),
            input_amount,
        );
        Self::transfer(
            exchange_a_address.clone(),
            inherent_asset_id.clone(),
            exchange_b_address.clone(),
            inherent_asset_output_amount,
        );
        Self::transfer(
            exchange_b_address.clone(),
            asset_b.clone(),
            output_account,
            asset_b_output_amount,
        );

        // debug
        runtime_io::print(
            "----> exchange a inherent asset balance, exchange a paired asset balance",
        );
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_a_address.clone()))
                .into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(asset_a.clone(), exchange_a_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print(
            "----> exchange b inherent asset balance, exchange b paired asset balance",
        );
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_b_address.clone()))
                .into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(asset_b.clone(), exchange_b_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print("----> account asset a balance, account asset b balance");
        let b: u128 = Self::get_asset_balance(&(asset_a.clone(), input_account.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(asset_b.clone(), input_account.clone())).into();
        runtime_io::print(paired_b as u64);

        Self::deposit_event(RawEvent::AssetsSwapped(
            input_account,
            asset_a,
            input_amount,
            asset_b,
            asset_b_output_amount,
        ));

        // emit event
        let asset_a_balance_in_pool = Self::balance(asset_a, exchange_a_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(asset_a, asset_a_balance_in_pool));
        let inherent_asset_balance_in_a_pool =
            Self::balance(inherent_asset_id, exchange_a_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_a_pool,
        ));
        let asset_b_balance_in_pool = Self::balance(asset_b, exchange_b_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(asset_b, asset_b_balance_in_pool));
        let inherent_asset_balance_in_pool_b =
            Self::balance(inherent_asset_id, exchange_b_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_pool_b,
        ));

        Ok(asset_b_output_amount)
    }

    /// Asset a to asset b, with exact output amount
    /// @input_account    The account to send asset a
    /// @output_account   The account to receive asset b
    /// @asset_a          The id of asset a
    /// @asset_b          The id of asset b
    /// @output_amount    The amount of output asset b
    /// @max_input_amount    The maximum amount of input asset a
    /// @fee_rate         The fee rate used to calculate the handing charge
    fn asset_a_to_asset_b_with_exact_output(
        input_account: T::AccountId,
        output_account: T::AccountId,
        asset_a: T::AssetId,
        asset_b: T::AssetId,
        output_amount: T::Balance,
        max_input_amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_a_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_a);
        let exchange_b_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_b);

        let inherent_asset_input_amount =
            Self::calc_inherent_asset_input_amount(asset_b, output_amount, fee_rate)?;
        let asset_a_input_amount =
            Self::calc_paired_asset_input_amount(asset_a, inherent_asset_input_amount, fee_rate)?;

        // CHECKS

        // do transfer
        Self::transfer(
            input_account.clone(),
            asset_a.clone(),
            exchange_a_address.clone(),
            asset_a_input_amount,
        );
        Self::transfer(
            exchange_a_address.clone(),
            inherent_asset_id.clone(),
            exchange_b_address.clone(),
            inherent_asset_input_amount,
        );
        Self::transfer(
            exchange_b_address.clone(),
            asset_b.clone(),
            output_account,
            output_amount,
        );

        // debug
        runtime_io::print(
            "----> exchange a inherent asset balance, exchange a paired asset balance",
        );
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_a_address.clone()))
                .into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(asset_a.clone(), exchange_a_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print(
            "----> exchange b inherent asset balance, exchange b paired asset balance",
        );
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_b_address.clone()))
                .into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(asset_b.clone(), exchange_b_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print("----> account asset a balance, account asset b balance");
        let b: u128 = Self::get_asset_balance(&(asset_a.clone(), input_account.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(asset_b.clone(), input_account.clone())).into();
        runtime_io::print(paired_b as u64);

        Self::deposit_event(RawEvent::AssetsSwapped(
            input_account,
            asset_a,
            asset_a_input_amount,
            asset_b,
            output_amount,
        ));

        // emit event
        let asset_a_balance_in_pool = Self::balance(asset_a, exchange_a_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(asset_a, asset_a_balance_in_pool));
        let inherent_asset_balance_in_a_pool =
            Self::balance(inherent_asset_id, exchange_a_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_a_pool,
        ));
        let asset_b_balance_in_pool = Self::balance(asset_b, exchange_b_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(asset_b, asset_b_balance_in_pool));
        let inherent_asset_balance_in_pool_b =
            Self::balance(inherent_asset_id, exchange_b_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_pool_b,
        ));

        Ok(asset_a_input_amount)
    }

    /// Calculate how much amount of inherent asset should be input
    /// @asset_id    The paired asset id
    /// @amount      The amount of the paired asset output
    /// @fee_rate    The fee rate used to calculate the handing charge
    fn calc_inherent_asset_input_amount(
        asset_id: T::AssetId,
        amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        // ensure!(amount > Zero::zero(), "");

        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_id);

        let inherent_asset_balance =
            Self::balance(inherent_asset_id.clone(), exchange_address.clone());
        let paired_asset_balance = Self::balance(asset_id, exchange_address);

        let input_amount = Self::calc_input_at_known_output(
            amount,
            inherent_asset_balance,
            paired_asset_balance,
            fee_rate,
        )?;

        Ok(input_amount)
    }

    /// Calculate how much amount of inherent asset should be output
    /// @asset_id    The paired asset id
    /// @amount      The amount of the paired asset input
    /// @fee_rate    The fee rate used to calculate the handing charge
    fn calc_inherent_asset_output_amount(
        asset_id: T::AssetId,
        amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        // ensure!(amount > Zero::zero(), "");

        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_id);

        let inherent_asset_balance = Self::balance(inherent_asset_id, exchange_address.clone());
        let paired_asset_balance = Self::balance(asset_id, exchange_address);

        let output_amount = Self::calc_output_at_known_input(
            amount,
            paired_asset_balance,
            inherent_asset_balance,
            fee_rate,
        )?;

        Ok(output_amount)
    }

    /// Calculate how much amount of paired asset should be input
    /// @asset_id    The paired asset id
    /// @amount      The amount of the inherent asset output
    /// @fee_rate    The fee rate used to calculate the handing charge
    fn calc_paired_asset_input_amount(
        asset_id: T::AssetId,
        amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        // ensure!(amount > Zero::zero(), "");

        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_id);

        let inherent_asset_balance = Self::balance(inherent_asset_id, exchange_address.clone());
        let paired_asset_balance = Self::balance(asset_id, exchange_address);

        let input_amount = Self::calc_input_at_known_output(
            amount,
            paired_asset_balance,
            inherent_asset_balance,
            fee_rate,
        )?;

        Ok(input_amount)
    }

    /// Calculate how much amount of paired asset should be output
    /// @asset_id    The paired asset id
    /// @amount      The amount of the inherent asset input
    /// @fee_rate    The fee rate used to calculate the handing charge
    fn calc_paired_asset_output_amount(
        asset_id: T::AssetId,
        amount: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        //ensure!(amount > Zero::zero(), "");

        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_id);

        let inherent_asset_balance = Self::balance(inherent_asset_id, exchange_address.clone());
        let paired_asset_balance = Self::balance(asset_id, exchange_address);

        let output_amount = Self::calc_output_at_known_input(
            amount,
            inherent_asset_balance,
            paired_asset_balance,
            fee_rate,
        )?;

        Ok(output_amount)
    }

    /// Given the exact known input, calculate the output
    /// @input_amount        The input asset amount
    /// @input_part_balance  The input asset balance in some paired pool
    /// @output_part_balance The output asset balance in some paired pool
    /// @fee_rate            The fee rate used to calculate the handing charge
    fn calc_output_at_known_input(
        input_amount: T::Balance,
        input_part_balance: T::Balance,
        output_part_balance: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        if input_part_balance.is_zero() || output_part_balance.is_zero() {
            return Err("Empty Pool.");
        }

        // TODO: calculate with fee rate
        let input_volumn: u128 = input_amount.into();
        let input_part_volumn: u128 = input_part_balance.into();
        let output_part_volumn: u128 = output_part_balance.into();

        // XXX: check overflow
        let denominator: u128 = input_volumn + input_part_volumn;
        let output_volumn: u128 = output_part_volumn * input_volumn / denominator;

        Ok(output_volumn.into())
    }

    /// Give the exact known output, calculate the input
    /// @output_amount       The output asset amount
    /// @input_part_balance  The input asset balance in some paired pool
    /// @output_part_balance The output asset balance in some paired pool
    /// @fee_rate            The fee rate used to calculate the handing charge
    fn calc_input_at_known_output(
        output_amount: T::Balance,
        input_part_balance: T::Balance,
        output_part_balance: T::Balance,
        fee_rate: T::FeeRate,
    ) -> rstd::result::Result<T::Balance, &'static str> {
        if input_part_balance.is_zero() || output_part_balance.is_zero() {
            return Err("Empty Pool.");
        }

        if output_amount >= output_part_balance {
            return Ok(T::Balance::max_value());
        }

        let output_volumn: u128 = output_amount.into();
        let input_part_volumn: u128 = input_part_balance.into();
        let output_part_volumn: u128 = output_part_balance.into();

        // XXX: check overflow
        let denominator: u128 = output_part_volumn - output_volumn;
        let input_volumn = input_part_volumn * output_volumn / denominator;

        // TODO: calculate with fee rate

        Ok(input_volumn.into())
    }

    /// Add liquidity
    /// the value of liquidity is equal to the value of input inherent asset
    /// @account    The account to inject liquidity to some paired pool
    /// @asset_id   The paired asset, used to represent which paired pool to act on
    /// @inherent_asset_amount    The amount of inherent asset part to be injected
    /// @asset_amount    The amount of paired asset part to be injected
    /// @min_liquidity   The limitation of minimum liquidity injected this time
    fn _add_liquidity(
        account: T::AccountId,
        asset_id: T::AssetId,
        inherent_asset_amount: T::Balance,
        asset_amount: T::Balance,
        min_liquidity: T::Balance,
    ) {
        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address = Self::get_exchange_address(inherent_asset_id, asset_id);

        // TODO: checks

        let total_liquidity = Self::get_total_liquidity(asset_id);

        if total_liquidity.is_zero() {
            // initializing injection
            Self::transfer(
                account.clone(),
                inherent_asset_id.clone(),
                exchange_address.clone(),
                inherent_asset_amount,
            );
            Self::transfer(
                account.clone(),
                asset_id.clone(),
                exchange_address.clone(),
                asset_amount,
            );

            Self::set_liquidity(asset_id.clone(), account.clone(), inherent_asset_amount);
            Self::increase_total_liquidity(asset_id.clone(), inherent_asset_amount);

        // emit event
        } else {
            let inherent_asset_in_pool =
                Self::balance(inherent_asset_id.clone(), exchange_address.clone());
            let asset_in_pool = Self::balance(asset_id, exchange_address.clone());

            Self::transfer(
                account.clone(),
                inherent_asset_id.clone(),
                exchange_address.clone(),
                inherent_asset_amount,
            );
            Self::transfer(
                account.clone(),
                asset_id.clone(),
                exchange_address.clone(),
                asset_amount,
            );

            // TODO: type convertions
            let minted_liquidity = total_liquidity * inherent_asset_amount / inherent_asset_in_pool;

            let pool_liquidity = Self::get_liquidity(asset_id.clone(), account.clone());
            Self::set_liquidity(
                asset_id.clone(),
                account.clone(),
                pool_liquidity + minted_liquidity,
            );
            Self::increase_total_liquidity(asset_id.clone(), minted_liquidity);
        }
        // update this key pair on every adding liquidity, no problem
        <ExchangeAccounts<T>>::insert(asset_id.clone(), exchange_address.clone());

        // debug
        runtime_io::print("----> Paired pool account liquidity");
        let b: u128 = Self::account_liquidity(&(asset_id.clone(), account.clone())).into();
        runtime_io::print(b as u64);
        runtime_io::print("----> Paired pool total liquidity");
        let b: u128 = Self::total_liquidity(asset_id.clone()).into();
        runtime_io::print(b as u64);

        runtime_io::print("----> exchange inherent asset balance, exchange paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print("----> account inherent asset balance, account paired asset balance");
        let b: u128 = Self::get_asset_balance(&(inherent_asset_id.clone(), account.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 = Self::get_asset_balance(&(asset_id.clone(), account.clone())).into();
        runtime_io::print(paired_b as u64);

        // emit event
        let asset_balance_in_pool = Self::balance(asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(asset_id, asset_balance_in_pool));
        let inherent_asset_balance_in_pool =
            Self::balance(inherent_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_pool,
        ));
    }

    /// Remove liquidity
    /// @account    The account to do removing liquidity from some paired pool
    /// @asset_id   The paired asset, used to represent which paired pool to act on
    /// @liquidity  The amount of liquidity to be removed
    /// @min_inherent_asset_amount    The minimum amount of inherent asset to be removed, used to check
    /// @min_asset_amount   The minimum amount of paired asset to be removed, used to check
    fn _remove_liquidity(
        account: T::AccountId,
        asset_id: T::AssetId,
        liquidity: T::Balance,
        min_inherent_asset_amount: T::Balance,
        min_asset_amount: T::Balance,
    ) {
        let inherent_asset_id = Self::inherent_asset_id();
        let exchange_address = Self::get_exchange_address(inherent_asset_id.clone(), asset_id);
        let account_liquidity = Self::get_liquidity(asset_id, account.clone());
        let total_liquidity = Self::get_total_liquidity(asset_id);

        let inherent_asset_in_pool = Self::balance(inherent_asset_id, exchange_address.clone());
        let asset_in_pool = Self::balance(asset_id, exchange_address.clone());

        // TODO: type cast
        let inherent_asset_amount = inherent_asset_in_pool * liquidity / total_liquidity;
        let asset_amount = asset_in_pool * liquidity / total_liquidity;

        // TODO: checks

        Self::transfer(
            exchange_address.clone(),
            inherent_asset_id.clone(),
            account.clone(),
            inherent_asset_amount,
        );
        Self::transfer(
            exchange_address.clone(),
            asset_id,
            account.clone(),
            asset_amount,
        );

        Self::set_liquidity(asset_id, account.clone(), account_liquidity - liquidity);
        Self::decrease_total_liquidity(asset_id, liquidity);

        // debug
        runtime_io::print("----> Paired pool account liquidity");
        let b: u128 = Self::account_liquidity(&(asset_id.clone(), account.clone())).into();
        runtime_io::print(b as u64);
        runtime_io::print("----> Paired pool total liquidity");
        let b: u128 = Self::total_liquidity(asset_id.clone()).into();
        runtime_io::print(b as u64);

        runtime_io::print("----> exchange inherent asset balance, exchange paired asset balance");
        let b: u128 =
            Self::get_asset_balance(&(inherent_asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 =
            Self::get_asset_balance(&(asset_id.clone(), exchange_address.clone())).into();
        runtime_io::print(paired_b as u64);

        runtime_io::print("----> account inherent asset balance, account paired asset balance");
        let b: u128 = Self::get_asset_balance(&(inherent_asset_id.clone(), account.clone())).into();
        runtime_io::print(b as u64);
        let paired_b: u128 = Self::get_asset_balance(&(asset_id.clone(), account.clone())).into();
        runtime_io::print(paired_b as u64);

        // emit event
        let asset_balance_in_pool = Self::balance(asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(asset_id, asset_balance_in_pool));
        let inherent_asset_balance_in_pool =
            Self::balance(inherent_asset_id, exchange_address.clone());
        Self::deposit_event(RawEvent::ReserveChanged(
            inherent_asset_id,
            inherent_asset_balance_in_pool,
        ));
    }

    /// Set liquidity of an account in a pool
    /// @asset_id   The paired asset, used to represent which paired pool to act on
    /// @account    The related account(owner) to this liquidity
    /// @liquidity  The liquidity value
    fn set_liquidity(asset_id: T::AssetId, account: T::AccountId, liquidity: T::Balance) {
        <AccountLiquidities<T>>::insert(&(asset_id.clone(), account.clone()), &liquidity);

        // debug
        runtime_io::print("----> Paired pool account liquidity");
        let b: u128 = Self::account_liquidity(&(asset_id.clone(), account.clone())).into();
        runtime_io::print(b as u64);
    }

    /// Get the liquidity of an account in a pool
    /// @asset_id   The paired asset, used to represent which paired pool to act on
    /// @account    The related account(owner) to wanted liquidity
    fn get_liquidity(asset_id: T::AssetId, account: T::AccountId) -> T::Balance {
        // debug
        runtime_io::print("----> Paired pool account liquidity");
        let b: u128 = Self::account_liquidity(&(asset_id.clone(), account.clone())).into();
        runtime_io::print(b as u64);

        <AccountLiquidities<T>>::get(&(asset_id, account))
    }

    /// Increase the total liquidity of a pool
    /// @asset_id   The paired asset, used to represent which paired pool to act on
    /// @liquidity  The liquidity value
    fn increase_total_liquidity(asset_id: T::AssetId, liquidity: T::Balance) {
        // XXX: check overflow
        <TotalLiquidities<T>>::mutate(asset_id, |b| *b += liquidity);

        // debug
        runtime_io::print("----> Paired pool total liquidity");
        let b: u128 = Self::total_liquidity(asset_id.clone()).into();
        runtime_io::print(b as u64);
    }

    /// Decrease the total liquidity of a pool
    /// @asset_id   The paired asset, used to represent which paired pool to act on
    /// @liquidity  The liquidity value
    fn decrease_total_liquidity(asset_id: T::AssetId, liquidity: T::Balance) {
        // XXX: check belowflow
        <TotalLiquidities<T>>::mutate(asset_id, |b| *b -= liquidity);

        // debug
        runtime_io::print("----> Paired pool total liquidity");
        let b: u128 = Self::total_liquidity(asset_id.clone()).into();
        runtime_io::print(b as u64);
    }

    /// Get the total liquidity of a pool
    /// @asset_id   The paired asset, used to represent which paired pool to act on
    fn get_total_liquidity(asset_id: T::AssetId) -> T::Balance {
        // debug
        runtime_io::print("----> Paired pool total liquidity");
        let b: u128 = Self::total_liquidity(asset_id.clone()).into();
        runtime_io::print(b as u64);

        <TotalLiquidities<T>>::get(asset_id)
    }

    /// Generate a new exchagne address (AccountId)
    /// @inherent_asset_id   Inherent asset id
    /// @asset_id    Paired asset id
    fn get_exchange_address(inherent_asset_id: T::AssetId, asset_id: T::AssetId) -> T::AccountId {
        T::ExchangeAddress::make_exchange_address(inherent_asset_id, asset_id)
    }

    /// Get the asset `id` balance of `who`.
    /// @id    Asset id
    /// @who   Account id
    pub fn balance(id: T::AssetId, who: T::AccountId) -> T::Balance {
        // debug
        runtime_io::print("----> Account Asset Balance");
        let b: u128 = Self::get_asset_balance(&(id.clone(), who.clone())).into();
        runtime_io::print(b as u64);

        <Balances<T>>::get((id, who))
    }

    /// Get the total supply of an asset `id`.
    /// @id    Asset id
    pub fn total_supply(id: T::AssetId) -> T::Balance {
        // debug
        runtime_io::print("----> Asset Total Supply");
        let b: u128 = Self::get_asset_total_supply(id.clone()).into();
        runtime_io::print(b as u64);

        <TotalSupply<T>>::get(id)
    }
}

/// Exchange Factory
pub trait ExchangeFactory<TAssetId: Sized, TAccountId: Sized> {
    /// The generate function
    fn make_exchange_address(inherent_asset_id: TAssetId, asset_id: TAssetId) -> TAccountId;
}

/// Exchange Address
pub struct ExchangeAddress<T: Trait>(PhantomData<T>);

/// Impl ExchangeFactory for ExchangeAddress
impl<T: Trait> ExchangeFactory<T::AssetId, T::AccountId> for ExchangeAddress<T>
where
    T::AccountId: UncheckedFrom<T::Hash>,
    u64: core::convert::From<<T as Trait>::AssetId>,
{
    fn make_exchange_address(inherent_asset_id: T::AssetId, asset_id: T::AssetId) -> T::AccountId {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"substrate-uniswap:");
        buf.extend_from_slice(&u64_to_bytes(inherent_asset_id.into()));
        buf.extend_from_slice(&u64_to_bytes(asset_id.into()));

        T::Hashing::hash(&buf[..]).unchecked_into()
    }
}

/// helper function
fn u64_to_bytes(x: u64) -> [u8; 8] {
    unsafe { mem::transmute(x.to_le()) }
}

/// Exchange Address for mock
pub struct ExchangeAddressMock<T: Trait>(PhantomData<T>);

/// Impl ExchangeFactory for ExchangeAddress
impl<T: Trait> ExchangeFactory<T::AssetId, T::AccountId> for ExchangeAddressMock<T>
where
    u64: core::convert::From<<T as Trait>::AssetId>,
    <T as system::Trait>::AccountId: core::convert::From<u64>,
{
    fn make_exchange_address(inherent_asset_id: T::AssetId, asset_id: T::AssetId) -> T::AccountId {
        let aid = 10000 + u64::from(asset_id);
        aid.into()
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use sr_primitives::weights::Weight;
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_ok, impl_outer_origin, parameter_types};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    impl Trait for Test {
        type Event = ();
        type Balance = u128;
        type AssetId = u64;
        type ExchangeAddress = ExchangeAddressMock<Self>;
        type FeeRate = u64;
    }

    type Us = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        let (mut conf, child_conf) = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let b = GenesisConfig::<Test> {
            owner: 1,
            fee_rate: 0,
        }
        .build_storage()
        .unwrap()
        .0;

        conf.extend(b);

        (conf, child_conf).into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            // assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            //assert_eq!(Superfluid::fee_rate(), 0);
            assert_eq!(0, 0);
        });
    }

    #[test]
    fn issue_asset() {
        with_externalities(&mut new_test_ext(), || {
            let origin = Origin::signed(1);
            let total_supply = 10000;
            Us::issue(origin, total_supply);
            let id = Us::next_asset_id() - 1;
            let t = Us::get_asset_total_supply(id);
            assert_eq!(t, total_supply);
            assert_ne!(t, total_supply + 1);
        });
    }

    #[test]
    fn issue_asset2() {
        with_externalities(&mut new_test_ext(), || {
            let origin = Origin::signed(1);
            let total_supply = 50000;
            Us::issue(origin, total_supply);
            let id = Us::next_asset_id() - 1;
            let t = Us::get_asset_total_supply(id);
            assert_eq!(t, total_supply);
            assert_ne!(t, total_supply + 1);
        });
    }

    #[test]
    fn test_uniswap_runtime() {
        with_externalities(&mut new_test_ext(), || {
            let user = 1;
            let origin = Origin::signed(user);

            // issue first asset
            let total_supply = 10000;
            Us::issue(origin.clone(), total_supply);
            let id = Us::next_asset_id() - 1;
            let t = Us::get_asset_total_supply(id);
            assert_eq!(t, total_supply);
            let b = Us::get_asset_balance(&(id, user));
            assert_eq!(b, total_supply);

            // issue second asset
            let total_supply = 50000;
            Us::issue(origin.clone(), total_supply);
            let id = Us::next_asset_id() - 1;
            let t = Us::get_asset_total_supply(id);
            assert_eq!(t, total_supply);
            let b = Us::get_asset_balance(&(id, user));
            assert_eq!(b, total_supply);

            // issue third asset
            let total_supply = 80000;
            Us::issue(origin.clone(), total_supply);
            let id = Us::next_asset_id() - 1;
            let t = Us::get_asset_total_supply(id);
            assert_eq!(t, total_supply);
            let b = Us::get_asset_balance(&(id, user));
            assert_eq!(b, total_supply);

            // set inherent asset
            Us::set_inherent_asset(origin.clone(), 0);
            let inherent_asset = Us::inherent_asset_id();
            assert_eq!(inherent_asset, 0);

            // add liquidity to 0-1 paired pool
            let paired_asset = 1;
            Us::add_liquidity(origin.clone(), paired_asset, 1000, 5000, 0);
            let tl = Us::total_liquidity(paired_asset);
            assert_eq!(tl, 1000);
            let al = Us::account_liquidity(&(paired_asset, user));
            assert_eq!(al, 1000);
            let exchange_address = Us::get_exchange_address(inherent_asset, paired_asset);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address));
            assert_eq!(iabe, 1000);
            let pabe = Us::get_asset_balance(&(paired_asset, exchange_address));
            assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            assert_eq!(iabu, 9000);
            let pabu = Us::get_asset_balance(&(paired_asset, user));
            assert_eq!(pabu, 45000);

            // swap inherent asset and paired asset
            Us::swap_assets_with_exact_input(
                origin.clone(),
                user,
                inherent_asset,
                paired_asset,
                100,
                0,
            );
            let exchange_address = Us::get_exchange_address(inherent_asset, paired_asset);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address));
            assert_eq!(iabe, 1100);
            let pabe = Us::get_asset_balance(&(paired_asset, exchange_address));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            assert_eq!(iabu, 8900);
            let pabu = Us::get_asset_balance(&(paired_asset, user));
            //assert_eq!(pabu, 45000);

            Us::swap_assets_with_exact_output(
                origin.clone(),
                user,
                inherent_asset,
                paired_asset,
                1000,
                1000,
            );
            let exchange_address = Us::get_exchange_address(inherent_asset, paired_asset);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address));
            //assert_eq!(iabe, 1100);
            let pabe = Us::get_asset_balance(&(paired_asset, exchange_address));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 8900);
            let pabu = Us::get_asset_balance(&(paired_asset, user));
            //assert_eq!(pabu, 45000);

            Us::swap_assets_with_exact_input(
                origin.clone(),
                user,
                paired_asset,
                inherent_asset,
                500,
                0,
            );
            let exchange_address = Us::get_exchange_address(inherent_asset, paired_asset);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address));
            //assert_eq!(iabe, 1100);
            let pabe = Us::get_asset_balance(&(paired_asset, exchange_address));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 8900);
            let pabu = Us::get_asset_balance(&(paired_asset, user));
            //assert_eq!(pabu, 45000);

            Us::swap_assets_with_exact_output(
                origin.clone(),
                user,
                paired_asset,
                inherent_asset,
                200,
                2000,
            );
            let exchange_address = Us::get_exchange_address(inherent_asset, paired_asset);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address));
            //assert_eq!(iabe, 1100);
            let pabe = Us::get_asset_balance(&(paired_asset, exchange_address));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 8900);
            let pabu = Us::get_asset_balance(&(paired_asset, user));
            //assert_eq!(pabu, 45000);

            // Swap asset a and asset b

            let asset_a = 1;
            let asset_b = 2;

            Us::add_liquidity(origin.clone(), asset_b, 1000, 8000, 0);
            let tl = Us::total_liquidity(asset_b);
            assert_eq!(tl, 1000);
            let al = Us::account_liquidity(&(asset_b, user));
            assert_eq!(al, 1000);
            let exchange_address = Us::get_exchange_address(inherent_asset, asset_b);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address));
            assert_eq!(iabe, 1000);
            let pabe = Us::get_asset_balance(&(asset_b, exchange_address));
            assert_eq!(pabe, 8000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 9000);
            let pabu = Us::get_asset_balance(&(asset_b, user));
            assert_eq!(pabu, 72000);

            Us::swap_assets_with_exact_input(origin.clone(), user, asset_a, asset_b, 100, 0);
            let exchange_address_a = Us::get_exchange_address(inherent_asset, asset_a);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address_a));
            //assert_eq!(iabe, 1100);
            let pabe = Us::get_asset_balance(&(asset_a, exchange_address_a));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 8900);
            let pabu = Us::get_asset_balance(&(asset_a, user));
            //assert_eq!(pabu, 45000);
            let exchange_address_b = Us::get_exchange_address(inherent_asset, asset_b);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address_b));
            //assert_eq!(iabe, 1100);
            let pabe = Us::get_asset_balance(&(asset_b, exchange_address_b));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 8900);
            let pabu = Us::get_asset_balance(&(asset_b, user));
            //assert_eq!(pabu, 45000);

            // swap inherent asset and paired asset
            Us::swap_assets_with_exact_output(origin.clone(), user, asset_a, asset_b, 200, 1000);
            let exchange_address_a = Us::get_exchange_address(inherent_asset, asset_a);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address_a));
            //assert_eq!(iabe, 1100);
            let pabe = Us::get_asset_balance(&(asset_a, exchange_address_a));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 8900);
            let pabu = Us::get_asset_balance(&(asset_a, user));
            //assert_eq!(pabu, 45000);
            let exchange_address_b = Us::get_exchange_address(inherent_asset, asset_b);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address_b));
            //assert_eq!(iabe, 1100);
            let pabe = Us::get_asset_balance(&(asset_b, exchange_address_b));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 8900);
            let pabu = Us::get_asset_balance(&(asset_b, user));
            //assert_eq!(pabu, 45000);

            // test remove_liquidity
            let paired_asset = 1;
            Us::remove_liquidity(origin.clone(), paired_asset, 500, 0, 0);
            let tl = Us::total_liquidity(paired_asset);
            assert_eq!(tl, 500);
            let al = Us::account_liquidity(&(paired_asset, user));
            assert_eq!(al, 500);
            let exchange_address = Us::get_exchange_address(inherent_asset, paired_asset);
            let iabe = Us::get_asset_balance(&(inherent_asset, exchange_address));
            //assert_eq!(iabe, 1000);
            let pabe = Us::get_asset_balance(&(paired_asset, exchange_address));
            //assert_eq!(pabe, 5000);
            let iabu = Us::get_asset_balance(&(inherent_asset, user));
            //assert_eq!(iabu, 9000);
            let pabu = Us::get_asset_balance(&(paired_asset, user));
            //assert_eq!(pabu, 45000);

            // Placeholder here for display debug info of above asserts
            // If want to see these debug info, please open it
            // assert_eq!(0, 1);
        });
    }

}
