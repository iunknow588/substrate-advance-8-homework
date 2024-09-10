
//! Autogenerated weights for `pallet_poe`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-09-09, STEPS: `20`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `lion`, CPU: `11th Gen Intel(R) Core(TM) i5-11320H @ 3.20GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/production/solochain-template-node
// benchmark
// pallet
// --chain
// dev
// --wasm-execution=compiled
// --pallet
// pallet_poe
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --output
// pallets/poe/src/weights.rs
// --template
// .maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_poe`.
pub trait WeightInfo {
	fn create_claim(b: u32, ) -> Weight;
	fn revoke_claim(b: u32, ) -> Weight;
	fn transfer_claim(b: u32, ) -> Weight;
}

/// Weights for `pallet_poe` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `PoeModule::Proofs` (r:1 w:1)
	/// Proof: `PoeModule::Proofs` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 4]`.
	fn create_claim(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `6`
		//  Estimated: `3522`
		// Minimum execution time: 8_724_000 picoseconds (皮秒).
		// 1 纳秒 = 1000 皮秒
		Weight::from_parts(14_603_599, 3522)	// 基本权重 + 
			.saturating_add(T::DbWeight::get().reads(1_u64)) // 1次数据库读取
			.saturating_add(T::DbWeight::get().writes(1_u64))	// 1次数据库写入
	}
	/// Storage: `PoeModule::Proofs` (r:1 w:1)
	/// Proof: `PoeModule::Proofs` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 4]`.
	fn revoke_claim(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `85 + b * (1 ±0)`
		//  Estimated: `3522`
		// Minimum execution time: 9_760_000 picoseconds.
		Weight::from_parts(11_441_135, 3522)
			// Standard Error: 241_968
			.saturating_add(Weight::from_parts(780_305, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `PoeModule::Proofs` (r:1 w:1)
	/// Proof: `PoeModule::Proofs` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 4]`.
	fn transfer_claim(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `85 + b * (1 ±0)`
		//  Estimated: `3522`
		// Minimum execution time: 10_316_000 picoseconds.
		Weight::from_parts(14_767_547, 3522)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `PoeModule::Proofs` (r:1 w:1)
	/// Proof: `PoeModule::Proofs` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 4]`.
	fn create_claim(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `6`
		//  Estimated: `3522`
		// Minimum execution time: 8_724_000 picoseconds.
		Weight::from_parts(14_603_599, 3522)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `PoeModule::Proofs` (r:1 w:1)
	/// Proof: `PoeModule::Proofs` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 4]`.
	fn revoke_claim(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `85 + b * (1 ±0)`
		//  Estimated: `3522`
		// Minimum execution time: 9_760_000 picoseconds.
		Weight::from_parts(11_441_135, 3522)
			// Standard Error: 241_968
			.saturating_add(Weight::from_parts(780_305, 0).saturating_mul(b.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `PoeModule::Proofs` (r:1 w:1)
	/// Proof: `PoeModule::Proofs` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 4]`.
	fn transfer_claim(_b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `85 + b * (1 ±0)`
		//  Estimated: `3522`
		// Minimum execution time: 10_316_000 picoseconds.
		Weight::from_parts(14_767_547, 3522)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}