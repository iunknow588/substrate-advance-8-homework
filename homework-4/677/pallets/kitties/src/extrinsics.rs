use frame_support::pallet_macros::pallet_section;

/// Define all extrinsics for the pallet.
#[pallet_section]
mod dispatches {
    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create())]
        pub fn create(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let value = Self::random_value(&who);
            Self::mint_kitty(&who, value)?;
            Ok(())
            //要獲得下一個kittty id
            /*
            let kitty_id = NextKittyId::<T>::get();

            // map: 需要包含kitty_id, kitty的值
            // Kitty 定義在lib.rs
            Kitties::<T>::insert(
                kitty_id, 
                Kitty{
                    dna:_value,
                    price: Some(0),
                }
            );

            // 需要將kitty id 加 1, 在執行之前需要做一個檢查
            // 目的是避免超出id上限
            // 要記得更新error.rs (加上KittyIdOverflow)
            let next_kitty_id = kitty_id.checked_add(1).ok_or(Error::<T>::KittyIdOverflow)?;
            
            // 將kitty id 存在 NextKittyId
            NextKittyId::<T>::put(next_kitty_id);
            KittyOwner::<T>::insert(kitty_id, _who.clone());

            // 要確保有在events.rs中定義KittyCreated
            /*
            KittyCreated {
                creator: T::AccountId,
                index: u64,
                data: [u8; 16],
            },
            */

            // 存在鏈上的event
            Self::deposit_event(Event::KittyCreated { 
                creator: _who,
                index: kitty_id,
                data: _value,
            });
            
            Ok(())
            */
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::breed())]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: u32, kitty_id_2: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentId);
            // let (Kitty(kitty_1), Kitty(kitty_2)) = (//Version 0
            let (Kitty { dna: kitty_1, .. }, Kitty { dna: kitty_2, .. }) = (
                Self::kitties(kitty_id_1).ok_or(Error::<T>::KittyNotExist)?,
                Self::kitties(kitty_id_2).ok_or(Error::<T>::KittyNotExist)?,
            );
            ensure!(
                Self::kitty_owner(kitty_id_1).as_ref() == Some(&who),
                Error::<T>::NotOwner
            );
            ensure!(
                Self::kitty_owner(kitty_id_2).as_ref() == Some(&who),
                Error::<T>::NotOwner
            );
            let value = Self::breed_kitty(&who, kitty_1, kitty_2);
            Self::mint_kitty(&who, value)?;
            Ok(())
        }
        /*
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, kitty_id: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let owner = KittyOwner::<T>::get(kitty_id).expect("");

            ensure!(_who == owner, Error::<T>::NotOwner);

            KittyOwner::<T>::insert(kitty_id, to.clone());

            Self::deposit_event(Event::Transferred {
                from: _who,
                to,
                index: kitty_id,
            });

            Ok(())
        }

        pub fn sale(
            origin: OriginFor<T>,
            kitty_id: u32,
            until_block: BlockNumberFor<T>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            KittiesOnSale::<T>::insert(kitty_id, until_block);
            
            Ok(())
        }

        pub fn bid(origin: OriginFor<T>, kitty_id: u32, price: BalanceOf<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            T::Currency::reserve(&_who, price)?;

            KittiesBid::<T>::mutate(kitty_id, |bids| match bids {
                Some(bids) => {
                    bids.push((_who.clone(), price));
                }
                None => {
                    *bids = Some(vec![(_who.clone(), price)]);
                }
            });

            Ok(())
        }
        */

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::transfer())]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, kitty_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !KittiesBid::<T>::contains_key(kitty_id),
                Error::<T>::KittyAlreadyOnSale
            );
            Self::transfer_kitty(who, to, kitty_id)?;
            Ok(())
        }
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::sale())]
        pub fn sale(
            origin: OriginFor<T>,
            kitty_id: u32,
            until_block: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::kitty_owner(kitty_id).as_ref() == Some(&who),
                Error::<T>::NotOwner
            );
            ensure!(
                !KittiesBid::<T>::contains_key(kitty_id),
                Error::<T>::KittyAlreadyOnSale
            );
            ensure!(
                until_block >= <system::Pallet<T>>::block_number() + T::MinBidBlockSpan::get(),
                Error::<T>::BlockSpanTooSmall
            );

            KittiesOnSale::<T>::try_append(&until_block, kitty_id)
                .map_err(|_| Error::<T>::TooManyBidOnOneBlock)?;
            KittiesBid::<T>::insert(kitty_id, Option::<(T::AccountId, BalanceOf<T>)>::default());
            Self::deposit_event(Event::KittyOnSale {
                owner: who,
                kitty_id,
                until_block,
            });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::bid())]
        pub fn bid(origin: OriginFor<T>, kitty_id: u32, price: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Some(&who) != Self::kitty_owner(kitty_id).as_ref(),
                Error::<T>::BidForSelf
            );
            let last_bid =
                KittiesBid::<T>::try_get(kitty_id).map_err(|_| Error::<T>::KittyNotOnSale)?;
            let stake_amount = T::StakeAmount::get();
            if let Some((last_bidder, last_price)) = last_bid {
                ensure!(
                    price >= last_price + T::MinBidIncrement::get(),
                    Error::<T>::KittyBidLessThanTheSumOfLastPriceAndMinimumBidIncrement
                );
                T::Currency::unreserve(&last_bidder, last_price + stake_amount);
            } else {
                ensure!(
                    price >= T::MinBidAmount::get(),
                    Error::<T>::KittyBidLessThanOrMinimumBidAmount
                );
            }

            T::Currency::reserve(&who, price + stake_amount)
                .map_err(|_| Error::<T>::NotEnoughBalanceForBidAndStaking)?;
            KittiesBid::<T>::insert(kitty_id, Some((who.clone(), price)));
            Self::deposit_event(Event::KittyBid {
                bidder: who,
                kitty_id,
                price,
            });
            Ok(())
        }

        /// Submit new price to the list.
        ///
        /// This method is a public function of the module and can be called from within
        /// a transaction. It appends given `price` to current list of prices.
        /// In our example the `offchain worker` will create, sign & submit a transaction that
        /// calls this function passing the price.
        ///
        /// The transaction needs to be signed (see `ensure_signed`) check, so that the caller
        /// pays a fee to execute it.
        /// This makes sure that it's not easy (or rather cheap) to attack the chain by submitting
        /// excessive transactions, but note that it doesn't ensure the price oracle is actually
        /// working and receives (and provides) meaningful data.
        /// This example is not focused on correctness of the oracle itself, but rather its
        /// purpose is to showcase offchain worker capabilities.
        #[pallet::call_index(5)]
        #[pallet::weight({0})]
        pub fn submit_price(origin: OriginFor<T>, price: u32) -> DispatchResultWithPostInfo {
            // Retrieve sender of the transaction.
            let who = ensure_signed(origin)?;
            // Add the price to the on-chain list.
            Self::add_price(Some(who), price);
            Ok(().into())
        }

        /// Submit new price to the list via unsigned transaction.
        ///
        /// Works exactly like the `submit_price` function, but since we allow sending the
        /// transaction without a signature, and hence without paying any fees,
        /// we need a way to make sure that only some transactions are accepted.
        /// This function can be called only once every `T::UnsignedInterval` blocks.
        /// Transactions that call that function are de-duplicated on the pool level
        /// via `validate_unsigned` implementation and also are rendered invalid if
        /// the function has already been called in current "session".
        ///
        /// It's important to specify `weight` for unsigned calls as well, because even though
        /// they don't charge fees, we still don't want a single block to contain unlimited
        /// number of such transactions.
        ///
        /// This example is not focused on correctness of the oracle itself, but rather its
        /// purpose is to showcase offchain worker capabilities.
        #[pallet::call_index(6)]
        #[pallet::weight({0})]
        pub fn submit_price_unsigned(
            origin: OriginFor<T>,
            _block_number: BlockNumberFor<T>,
            price: u32,
        ) -> DispatchResultWithPostInfo {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Add the price to the on-chain list, but mark it as coming from an empty address.
            Self::add_price(None, price);
            // now increment the block number at which we expect next unsigned transaction.
            let current_block = <system::Pallet<T>>::block_number();
            <NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());
            Ok(().into())
        }

        #[pallet::call_index(7)]
        #[pallet::weight({0})]
        pub fn submit_price_unsigned_with_signed_payload(
            origin: OriginFor<T>,
            price_payload: PricePayload<T::Public, BlockNumberFor<T>>,
            _signature: T::Signature,
        ) -> DispatchResultWithPostInfo {
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Add the price to the on-chain list, but mark it as coming from an empty address.
            Self::add_price(None, price_payload.price);
            // now increment the block number at which we expect next unsigned transaction.
            let current_block = <system::Pallet<T>>::block_number();
            <NextUnsignedAt<T>>::put(current_block + T::UnsignedInterval::get());
            Ok(().into())
        }

    }
}