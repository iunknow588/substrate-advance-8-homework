use super::*;
use crate::{mock::*, Error, Event};
use frame_support::{
    assert_noop, assert_ok,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, ReservableCurrency},
};
use sp_core::{
    offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
    sr25519::Signature,
    H256,
};
use sp_keystore::{testing::MemoryKeystore, Keystore, KeystoreExt};
use sp_runtime::{
    testing::TestXt,
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
    RuntimeAppPublic,
};
#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        run_to_block(2);
    });
}
#[test]
fn it_works_for_sale() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        let (owner, bidder, kitty_id, price, until_block) = (alice(), bob(), 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));

        // sale kitty & with price
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        assert_eq!(
            KittiesOnSale::<Test>::get(&until_block),
            BoundedVec::<u32, <Test as Config>::MaxKittiesBidPerBlock>::try_from(vec![kitty_id])
                .unwrap()
        );

        run_to_block(2);

        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));
        let origin_reserved_balance_of_owner = <Test as Config>::Currency::reserved_balance(&owner);
        let origin_free_balance_of_owner = <Test as Config>::Currency::free_balance(&owner);
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&bidder);
        let origin_free_balance = <Test as Config>::Currency::free_balance(&bidder);
        run_to_block(until_block);
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(bidder));
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&bidder),
            origin_reserved_balance - price
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&bidder),
            origin_free_balance
        );
        let stake_amount = <<Test as Config>::StakeAmount as Get<u128>>::get();
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&owner),
            origin_reserved_balance_of_owner - stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&owner),
            origin_free_balance_of_owner + price + stake_amount
        );
        System::assert_has_event(
            Event::<Test>::KittyTransferredAfterBidKnockedDown {
                from: owner,
                to: bidder,
                kitty_id,
                price,
                usd_price: PalletKitties::average_price().map(|p| price * p as u128),
            }
            .into(),
        );
    });
}

#[test]
fn it_failed_for_sale_when_not_enough_balance() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        let (owner, bidder, kitty_id, price, until_block) = (alice(), dave(), 1, 500, 11);

        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));

        // sale kitty & with price 10
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        assert_eq!(
            KittiesOnSale::<Test>::get(&until_block),
            BoundedVec::<u32, <Test as Config>::MaxKittiesBidPerBlock>::try_from(vec![kitty_id])
                .unwrap()
        );

        run_to_block(2);

        let stake_amount = <<Test as Config>::StakeAmount as Get<u128>>::get();

        let _ = <Test as Config>::Currency::transfer(
            &owner,
            &bidder,
            <Test as Config>::Currency::minimum_balance() + price + stake_amount,
            ExistenceRequirement::KeepAlive,
        );
        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));

        <Test as Config>::Currency::unreserve(&bidder, stake_amount + 3);

        let _ = <Test as Config>::Currency::transfer(
            &bidder,
            &owner,
            stake_amount + 3,
            ExistenceRequirement::KeepAlive,
        );

        run_to_block(until_block);
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(owner));
    });
}

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id) = (alice(), 1);
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&creator);
        let origin_free_balance = <Test as Config>::Currency::free_balance(&creator);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_eq!(NextKittyId::<Test>::get(), kitty_id);
        assert!(Kitties::<Test>::get(kitty_id).is_some());
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(creator));
        let stake_amount = <<Test as Config>::StakeAmount as Get<u128>>::get();
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&creator),
            origin_reserved_balance + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&creator),
            origin_free_balance - stake_amount
        );
        System::assert_has_event(
            Event::<Test>::KittyCreated {
                creator,
                kitty_id,
                data: Kitties::<Test>::get(kitty_id).unwrap().dna.clone(),
            }
            .into(),
        );
    });
}

#[test]
fn create_failed_when_next_kitty_id_overflow() {
    new_test_ext().execute_with(|| {
        let creator = alice();
        NextKittyId::<Test>::put(u32::MAX);
        assert_noop!(
            PalletKitties::create(RuntimeOrigin::signed(creator)),
            Error::<Test>::NextKittyIdOverflow
        );
    });
}

#[test]
fn create_failed_when_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
        let creator = dave();
        assert_noop!(
            PalletKitties::create(RuntimeOrigin::signed(creator)),
            Error::<Test>::NotEnoughBalanceForStaking
        );
    });
}

#[test]
fn breed_works() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id_1, kitty_id_2, kitty_id) = (alice(), 1, 2, 3);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&creator);
        let origin_free_balance = <Test as Config>::Currency::free_balance(&creator);
        assert_ok!(PalletKitties::breed(
            RuntimeOrigin::signed(creator),
            kitty_id_1,
            kitty_id_2
        ));
        assert_eq!(NextKittyId::<Test>::get(), kitty_id);
        let stake_amount = <<Test as Config>::StakeAmount as Get<u128>>::get();
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&creator),
            origin_reserved_balance + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&creator),
            origin_free_balance - stake_amount
        );
        System::assert_has_event(
            Event::<Test>::KittyCreated {
                creator,
                kitty_id,
                data: Kitties::<Test>::get(kitty_id).unwrap().dna.clone(),
            }
            .into(),
        );
    });
}

#[test]
fn breed_faile_when_same_parent_id() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id_1) = (alice(), 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));

        assert_noop!(
            PalletKitties::breed(RuntimeOrigin::signed(creator), kitty_id_1, kitty_id_1),
            Error::<Test>::SameParentId
        );
    });
}

#[test]
fn breed_faile_when_kitty_parent1_not_owner() {
    new_test_ext().execute_with(|| {
        let (creator, other, kitty_id_1, kitty_id_2) = (alice(), bob(), 1, 2);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(other)));
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_noop!(
            PalletKitties::breed(RuntimeOrigin::signed(creator), kitty_id_1, kitty_id_2),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn breed_faile_when_kitty_parent2_not_owner() {
    new_test_ext().execute_with(|| {
        let (creator, other, kitty_id_1, kitty_id_2) = (alice(), bob(), 1, 2);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(other)));
        assert_noop!(
            PalletKitties::breed(RuntimeOrigin::signed(creator), kitty_id_1, kitty_id_2),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn breed_faile_when_kitty1_not_exist() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id_1, kitty_id_2) = (alice(), 1, 2);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_noop!(
            PalletKitties::breed(RuntimeOrigin::signed(creator), kitty_id_1, kitty_id_2),
            Error::<Test>::KittyNotExist
        );
    });
}
#[test]
fn breed_faile_when_kitty2_not_exist() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id_1, kitty_id_2) = (alice(), 1, 2);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_noop!(
            PalletKitties::breed(RuntimeOrigin::signed(creator), kitty_id_1, kitty_id_2),
            Error::<Test>::KittyNotExist
        );
    });
}
#[test]
fn breed_failed_when_next_kitty_id_overflow() {
    new_test_ext().execute_with(|| {
        let (creator, kitty_id_1, kitty_id_2) = (alice(), 1, 2);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        NextKittyId::<Test>::put(u32::MAX);
        assert_noop!(
            PalletKitties::breed(RuntimeOrigin::signed(creator), kitty_id_1, kitty_id_2),
            Error::<Test>::NextKittyIdOverflow
        );
    });
}

#[test]
fn breed_failed_when_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
        let (alice, creator, kitty_id_1, kitty_id_2) = (alice(), dave(), 1, 2);
        let _ = <Test as Config>::Currency::transfer(
            &alice,
            &creator,
            <Test as Config>::Currency::minimum_balance()
                + 2 * <<Test as Config>::StakeAmount as Get<u128>>::get(),
            ExistenceRequirement::KeepAlive,
        );
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(creator)));
        assert_noop!(
            PalletKitties::breed(RuntimeOrigin::signed(creator), kitty_id_1, kitty_id_2),
            Error::<Test>::NotEnoughBalanceForStaking
        );
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id) = (alice(), bob(), 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        let stake_amount = <<Test as Config>::StakeAmount as Get<u128>>::get();
        let origin_reserved_balance_of_from = <Test as Config>::Currency::reserved_balance(&from);
        let origin_reserved_balance_of_to = <Test as Config>::Currency::reserved_balance(&to);
        let origin_free_balance_of_from = <Test as Config>::Currency::free_balance(&from);
        let origin_free_balance_of_to = <Test as Config>::Currency::free_balance(&to);
        assert_ok!(PalletKitties::transfer(
            RuntimeOrigin::signed(from),
            to,
            kitty_id
        ));
        assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(to));

        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&from),
            origin_reserved_balance_of_from - stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&to),
            origin_reserved_balance_of_to + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&from),
            origin_free_balance_of_from + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&to),
            origin_free_balance_of_to - stake_amount
        );
        System::assert_has_event(Event::<Test>::KittyTransferred { from, to, kitty_id }.into());
    });
}

#[test]
fn transfer_failed_when_kitty_already_on_sale() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id, until_block) = (alice(), bob(), 1, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(from),
            kitty_id,
            until_block
        ));
        assert_noop!(
            PalletKitties::transfer(RuntimeOrigin::signed(from), to, kitty_id),
            Error::<Test>::KittyAlreadyOnSale
        );
    });
}

#[test]
fn transfer_failed_when_not_owner() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id) = (alice(), bob(), 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        assert_noop!(
            PalletKitties::transfer(RuntimeOrigin::signed(to), from, kitty_id),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn transfer_failed_when_not_enough_balance_for_staking() {
    new_test_ext().execute_with(|| {
        let (from, to, kitty_id) = (alice(), dave(), 1);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(from)));
        assert_noop!(
            PalletKitties::transfer(RuntimeOrigin::signed(from), to, kitty_id),
            Error::<Test>::NotEnoughBalanceForStaking
        );
    });
}

#[test]
fn sale_works() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, until_block) = (alice(), 1, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));

        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        assert_eq!(
            KittiesOnSale::<Test>::get(&until_block),
            BoundedVec::<u32, <Test as Config>::MaxKittiesBidPerBlock>::try_from(vec![kitty_id])
                .unwrap()
        );
        assert!(KittiesBid::<Test>::get(kitty_id).is_none());

        System::assert_has_event(
            Event::<Test>::KittyOnSale {
                owner,
                kitty_id,
                until_block,
            }
            .into(),
        );
    });
}

#[test]
fn sale_failed_when_not_owner() {
    new_test_ext().execute_with(|| {
        let (owner, other, kitty_id, until_block) = (alice(), bob(), 1, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_noop!(
            PalletKitties::sale(RuntimeOrigin::signed(other), kitty_id, until_block),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn sale_failed_when_kitty_already_on_sale() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, until_block) = (alice(), 1, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        assert_noop!(
            PalletKitties::sale(RuntimeOrigin::signed(owner), kitty_id, until_block),
            Error::<Test>::KittyAlreadyOnSale
        );
    });
}

#[test]
fn sale_failed_when_block_span_too_small() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, until_block) = (alice(), 1, 10);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_noop!(
            PalletKitties::sale(RuntimeOrigin::signed(owner), kitty_id, until_block),
            Error::<Test>::BlockSpanTooSmall
        );
    });
}

#[test]
fn sale_failed_when_too_many_bid_on_one_block() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, until_block) = (alice(), 11, 11);
        for id in 1..=10 {
            assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
            assert_ok!(PalletKitties::sale(
                RuntimeOrigin::signed(owner),
                id,
                until_block
            ));
        }
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_noop!(
            PalletKitties::sale(RuntimeOrigin::signed(owner), kitty_id, until_block),
            Error::<Test>::TooManyBidOnOneBlock
        );
    });
}

#[test]
fn bid_works() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price, until_block) = (alice(), bob(), 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&bidder);
        let origin_free_balance = <Test as Config>::Currency::free_balance(&bidder);
        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));
        let stake_amount = <<Test as Config>::StakeAmount as Get<u128>>::get();
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&bidder),
            origin_reserved_balance + price + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&bidder),
            origin_free_balance - price - stake_amount
        );
        System::assert_has_event(
            Event::<Test>::KittyBid {
                bidder,
                kitty_id,
                price,
            }
            .into(),
        );
    });
}

#[test]
fn bid_works_when_the_second_bidder() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, bidder2, kitty_id, price, until_block) =
            (alice(), bob(), carol(), 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&bidder);
        let origin_free_balance = <Test as Config>::Currency::free_balance(&bidder);
        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));
        let stake_amount = <<Test as Config>::StakeAmount as Get<u128>>::get();
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&bidder),
            origin_reserved_balance + price + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&bidder),
            origin_free_balance - price - stake_amount
        );
        System::assert_has_event(
            Event::<Test>::KittyBid {
                bidder,
                kitty_id,
                price,
            }
            .into(),
        );

        let (last_bidder, last_price) = (bidder, price);
        let (bidder, price) = (
            bidder2,
            last_price + <<Test as Config>::MinBidIncrement as Get<u128>>::get(),
        );
        let origin_reserved_balance = <Test as Config>::Currency::reserved_balance(&bidder);
        let origin_free_balance = <Test as Config>::Currency::free_balance(&bidder);
        let origin_reserved_balance_of_last_bidder =
            <Test as Config>::Currency::reserved_balance(&last_bidder);
        let origin_free_balance_of_last_bidder =
            <Test as Config>::Currency::free_balance(&last_bidder);
        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_eq!(KittiesBid::<Test>::get(kitty_id), Some((bidder, price)));
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&bidder),
            origin_reserved_balance + price + stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&bidder),
            origin_free_balance - price - stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::reserved_balance(&last_bidder),
            origin_reserved_balance_of_last_bidder - last_price - stake_amount
        );
        assert_eq!(
            <Test as Config>::Currency::free_balance(&last_bidder),
            origin_free_balance_of_last_bidder + last_price + stake_amount
        );
        System::assert_has_event(
            Event::<Test>::KittyBid {
                bidder,
                kitty_id,
                price,
            }
            .into(),
        );
    });
}

#[test]
fn bid_failed_when_bid_for_self() {
    new_test_ext().execute_with(|| {
        let (owner, kitty_id, price, until_block) = (alice(), 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));

        assert_noop!(
            PalletKitties::bid(RuntimeOrigin::signed(owner), kitty_id, price),
            Error::<Test>::BidForSelf
        );
    });
}

#[test]
fn bid_failed_when_kitty_not_on_sale() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price) = (alice(), bob(), 1, 500);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_noop!(
            PalletKitties::bid(RuntimeOrigin::signed(bidder), kitty_id, price),
            Error::<Test>::KittyNotOnSale
        );
    });
}

#[test]
fn bid_failed_when_kitty_bid_less_than_or_minimum_bid_amount() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price, until_block) = (alice(), dave(), 1, 2, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));

        assert_noop!(
            PalletKitties::bid(RuntimeOrigin::signed(bidder), kitty_id, price),
            Error::<Test>::KittyBidLessThanOrMinimumBidAmount
        );
    });
}

#[test]
fn bid_failed_when_kitty_bid_less_than_the_sum_of_last_price_and_minimum_bid_increment() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, bidder2, kitty_id, price, until_block) =
            (alice(), bob(), dave(), 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));
        assert_ok!(PalletKitties::bid(
            RuntimeOrigin::signed(bidder),
            kitty_id,
            price
        ));
        assert_noop!(
            PalletKitties::bid(RuntimeOrigin::signed(bidder2), kitty_id, price),
            Error::<Test>::KittyBidLessThanTheSumOfLastPriceAndMinimumBidIncrement
        );
    });
}

#[test]
fn bid_failed_when_not_enough_balance_for_bid_and_staking() {
    new_test_ext().execute_with(|| {
        let (owner, bidder, kitty_id, price, until_block) = (alice(), dave(), 1, 500, 11);
        assert_ok!(PalletKitties::create(RuntimeOrigin::signed(owner)));
        assert_ok!(PalletKitties::sale(
            RuntimeOrigin::signed(owner),
            kitty_id,
            until_block
        ));

        assert_noop!(
            PalletKitties::bid(RuntimeOrigin::signed(bidder), kitty_id, price),
            Error::<Test>::NotEnoughBalanceForBidAndStaking
        );
    });
}

fn alice() -> sp_core::sr25519::Public {
    sp_core::sr25519::Public::from_raw([1u8; 32])
}

fn bob() -> sp_core::sr25519::Public {
    sp_core::sr25519::Public::from_raw([2u8; 32])
}

fn carol() -> sp_core::sr25519::Public {
    sp_core::sr25519::Public::from_raw([3u8; 32])
}
fn dave() -> sp_core::sr25519::Public {
    sp_core::sr25519::Public::from_raw([4u8; 32])
}
#[test]
fn it_aggregates_the_price() {
    sp_io::TestExternalities::default().execute_with(|| {
        assert_eq!(PalletKitties::average_price(), None);

        assert_ok!(PalletKitties::submit_price(
            RuntimeOrigin::signed(alice()),
            27
        ));
        assert_eq!(PalletKitties::average_price(), Some(27));

        assert_ok!(PalletKitties::submit_price(
            RuntimeOrigin::signed(alice()),
            43
        ));
        assert_eq!(PalletKitties::average_price(), Some(35));
    });
}

#[test]
fn should_make_http_call_and_parse_result() {
    let (offchain, state) = testing::TestOffchainExt::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));

    price_oracle_response(&mut state.write());

    t.execute_with(|| {
        // when
        let price = PalletKitties::fetch_price().unwrap();
        // then
        assert_eq!(price, 15523);
    });
}

#[test]
fn knows_how_to_mock_several_http_calls() {
    let (offchain, state) = testing::TestOffchainExt::new();
    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));

    {
        let mut state = state.write();
        state.expect_request(testing::PendingRequest {
            method: "GET".into(),
            uri: "https://min-api.cryptocompare.com/data/price?fsym=DOT&tsyms=USD".into(),
            response: Some(br#"{"USD": 1}"#.to_vec()),
            sent: true,
            ..Default::default()
        });

        state.expect_request(testing::PendingRequest {
            method: "GET".into(),
            uri: "https://min-api.cryptocompare.com/data/price?fsym=DOT&tsyms=USD".into(),
            response: Some(br#"{"USD": 2}"#.to_vec()),
            sent: true,
            ..Default::default()
        });

        state.expect_request(testing::PendingRequest {
            method: "GET".into(),
            uri: "https://min-api.cryptocompare.com/data/price?fsym=DOT&tsyms=USD".into(),
            response: Some(br#"{"USD": 3}"#.to_vec()),
            sent: true,
            ..Default::default()
        });
    }

    t.execute_with(|| {
        let price1 = PalletKitties::fetch_price().unwrap();
        let price2 = PalletKitties::fetch_price().unwrap();
        let price3 = PalletKitties::fetch_price().unwrap();

        assert_eq!(price1, 100);
        assert_eq!(price2, 200);
        assert_eq!(price3, 300);
    })
}

#[test]
fn should_submit_signed_transaction_on_chain() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";

    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();
    let keystore = MemoryKeystore::new();
    keystore
        .sr25519_generate_new(
            crate::crypto::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    price_oracle_response(&mut offchain_state.write());

    t.execute_with(|| {
        // when
        PalletKitties::fetch_price_and_send_signed().unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature.unwrap().0, 0);
        assert_eq!(
            tx.call,
            RuntimeCall::PalletKitties(crate::Call::submit_price { price: 15523 })
        );
    });
}

#[test]
fn should_submit_unsigned_transaction_on_chain_for_any_account() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = MemoryKeystore::new();

    keystore
        .sr25519_generate_new(
            crate::crypto::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

    let public_key = *keystore
        .sr25519_public_keys(crate::crypto::Public::ID)
        .get(0)
        .unwrap();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    price_oracle_response(&mut offchain_state.write());

    let price_payload = PricePayload {
        block_number: 1,
        price: 15523,
        public: <Test as SigningTypes>::Public::from(public_key),
    };

    // let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
    t.execute_with(|| {
        // when
        PalletKitties::fetch_price_and_send_unsigned_for_any_account(1).unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature, None);
        if let RuntimeCall::PalletKitties(
            crate::Call::submit_price_unsigned_with_signed_payload {
                price_payload: body,
                signature,
            },
        ) = tx.call
        {
            assert_eq!(body, price_payload);

            let signature_valid = <PricePayload<
                <Test as SigningTypes>::Public,
                frame_system::pallet_prelude::BlockNumberFor<Test>,
            > as SignedPayload<Test>>::verify::<crypto::TestAuthId>(
                &price_payload, signature
            );

            assert!(signature_valid);
        }
    });
}

#[test]
fn should_submit_unsigned_transaction_on_chain_for_all_accounts() {
    const PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = MemoryKeystore::new();

    keystore
        .sr25519_generate_new(
            crate::crypto::Public::ID,
            Some(&format!("{}/hunter1", PHRASE)),
        )
        .unwrap();

    let public_key = *keystore
        .sr25519_public_keys(crate::crypto::Public::ID)
        .get(0)
        .unwrap();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    price_oracle_response(&mut offchain_state.write());

    let price_payload = PricePayload {
        block_number: 1,
        price: 15523,
        public: <Test as SigningTypes>::Public::from(public_key),
    };

    // let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
    t.execute_with(|| {
        // when
        PalletKitties::fetch_price_and_send_unsigned_for_all_accounts(1).unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature, None);
        if let RuntimeCall::PalletKitties(
            crate::Call::submit_price_unsigned_with_signed_payload {
                price_payload: body,
                signature,
            },
        ) = tx.call
        {
            assert_eq!(body, price_payload);

            let signature_valid = <PricePayload<
                <Test as SigningTypes>::Public,
                frame_system::pallet_prelude::BlockNumberFor<Test>,
            > as SignedPayload<Test>>::verify::<crypto::TestAuthId>(
                &price_payload, signature
            );

            assert!(signature_valid);
        }
    });
}

#[test]
fn should_submit_raw_unsigned_transaction_on_chain() {
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = testing::TestTransactionPoolExt::new();

    let keystore = MemoryKeystore::new();

    let mut t = sp_io::TestExternalities::default();
    t.register_extension(OffchainWorkerExt::new(offchain));
    t.register_extension(TransactionPoolExt::new(pool));
    t.register_extension(KeystoreExt::new(keystore));

    price_oracle_response(&mut offchain_state.write());

    t.execute_with(|| {
        // when
        PalletKitties::fetch_price_and_send_raw_unsigned(1).unwrap();
        // then
        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let tx = Extrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature, None);
        assert_eq!(
            tx.call,
            RuntimeCall::PalletKitties(crate::Call::submit_price_unsigned {
                block_number: 1,
                price: 15523
            })
        );
    });
}

fn price_oracle_response(state: &mut testing::OffchainState) {
    state.expect_request(testing::PendingRequest {
        method: "GET".into(),
        uri: "https://min-api.cryptocompare.com/data/price?fsym=DOT&tsyms=USD".into(),
        response: Some(br#"{"USD": 155.23}"#.to_vec()),
        sent: true,
        ..Default::default()
    });
}

#[test]
fn parse_price_works() {
    let test_data = alloc::vec![
        ("{\"USD\":6536.92}", Some(653692)),
        ("{\"USD\":65.92}", Some(6592)),
        ("{\"USD\":6536.924565}", Some(653692)),
        ("{\"USD\":6536}", Some(653600)),
        ("{\"USD2\":6536}", None),
        ("{\"USD\":\"6432\"}", None),
    ];

    for (json, expected) in test_data {
        assert_eq!(expected, PalletKitties::parse_price(json));
    }
}
