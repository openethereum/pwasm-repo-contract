#![cfg_attr(not(feature="std"), no_main)]
#![cfg_attr(not(feature="std"), no_std)]

#![feature(proc_macro)]
#![feature(alloc)]
#![allow(non_snake_case)]

extern crate alloc;
extern crate bigint;
extern crate parity_hash;
extern crate pwasm_std;
extern crate pwasm_ethereum;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;
extern crate pwasm_token_contract;

use pwasm_std::hash::{Address, H256};
use bigint::U256;
use pwasm_abi_derive::eth_abi;

use pwasm_token_contract::TokenContract;
use pwasm_token_contract::Client as Token;

use pwasm_ethereum as eth;
use pwasm_std::Vec;

// Generates storage keys. Each key = previous_key + 1. 256 keys max
macro_rules! storage_keys {
	() => {};
	($($name:ident),*) => {
		storage_keys!(0u8, $($name),*);
	};
	($count:expr, $name:ident) => {
		static $name: H256 = H256([$count, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
	};
	($count:expr, $name:ident, $($tail:ident),*) => {
		static $name: H256 = H256([$count, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
		storage_keys!($count + 1u8, $($tail),*);
	};
}

struct Entry {
	key: H256,
	value: [u8; 32]
}

struct Storage {
	table: Vec<Entry>,
}

/// TODO: optimise by impl hashtable
impl Storage {
	fn with_capacity(cap: usize) -> Storage {
		Storage {
			table: Vec::with_capacity(cap)
		}
	}

	fn read(&mut self, key: &H256) -> [u8; 32] {
		// First: lookup in the table
		for entry in &self.table {
			if *key == entry.key {
				return entry.value.clone();
			}
		}
		// Second: read from the storage
		let value = eth::read(key);
		self.table.push(Entry {
			key: key.clone(),
			value: value.clone()
		});
		value
	}

	fn write(&mut self, key: &H256, value: &[u8; 32]) {
		eth::write(key, value);
		for entry in &mut self.table {
			if *key == entry.key {
				entry.value = *value;
				return;
			}
		}
		self.table.push(Entry {
			key: key.clone(),
			value: value.clone()
		});
	}
}

#[eth_abi(Endpoint, Client)]
pub trait RepoContract {
	fn constructor(&mut self,
		borrower: Address,
		lender: Address,
		loan_token: Address,
		security_token: Address,
		loan_amount: U256,
		security_amount: U256,
		interest_rate: U256,
		activation_deadline: u64,
		return_deadline: u64);
	fn accept(&mut self) -> bool;
	fn terminate(&mut self) -> bool;
	#[event]
	fn LenderAcceptance(&mut self);
	#[event]
	fn BorrowerAcceptance(&mut self);
	#[event]
	fn Activation(&mut self);
	#[event]
	fn Refund(&mut self);
	#[event]
	fn Default(&mut self);
	#[event]
	fn Suicide(&mut self);
}

storage_keys!(
	BORROWER_KEY, LENDER_KEY,
	LOAN_TOKEN_KEY, SECURITY_TOKEN_KEY,
	LOAN_AMOUNT_KEY, SECURITY_AMOUNT_KEY,
	INTEREST_RATE_KEY,
	ACTIVATION_DEADLINE_KEY, RETURN_DEADLINE_KEY,
	BORROW_ACCEPTED_KEY, LEND_ACCEPTED_KEY
);

static DIVISOR: U256 = U256([100,0,0,0]);

pub struct RepoContractInstance {
	storage: Storage
}

impl RepoContractInstance {
	pub fn new() -> RepoContractInstance {
		RepoContractInstance {
			storage: Storage::with_capacity(10)
		}
	}
	pub fn borrower_address(&mut self) -> Address {
		H256::from(self.storage.read(&BORROWER_KEY)).into()
	}

	pub fn lender_address(&mut self) -> Address {
		H256::from(self.storage.read(&LENDER_KEY)).into()
	}

	pub fn loan_token_address(&mut self) -> Address {
		H256::from(self.storage.read(&LOAN_TOKEN_KEY)).into()
	}

	pub fn security_token_address(&mut self) -> Address {
		H256::from(self.storage.read(&SECURITY_TOKEN_KEY)).into()
	}

	pub fn loan_amount(&mut self) -> U256 {
		self.storage.read(&LOAN_AMOUNT_KEY).into()
	}

	pub fn security_amount(&mut self) -> U256 {
		self.storage.read(&SECURITY_AMOUNT_KEY).into()
	}

	pub fn interest_rate(&mut self) -> U256 {
		self.storage.read(&INTEREST_RATE_KEY).into()
	}

	// Activation deadline timestamp
	pub fn activation_deadline(&mut self) -> u64 {
		U256::from(self.storage.read(&ACTIVATION_DEADLINE_KEY)).into()
	}

	// Return deadline timestamp
	pub fn return_deadline(&mut self) -> u64 {
		U256::from(self.storage.read(&RETURN_DEADLINE_KEY)).into()
	}

	pub fn borrower_acceptance(&mut self) -> bool {
		let value = U256::from(self.storage.read(&BORROW_ACCEPTED_KEY));
		if value == 0.into() {
			false
		} else {
			true
		}
	}

	pub fn lender_acceptance(&mut self) -> bool {
		let value = U256::from(self.storage.read(&LEND_ACCEPTED_KEY));
		if value == 0.into() {
			false
		} else {
			true
		}
	}

	pub fn is_active(&mut self) -> bool {
		self.borrower_acceptance() && self.lender_acceptance()
	}

	pub fn suicide(&mut self, sender: &Address) -> bool {
		self.Suicide();
		if cfg!(test) {
			return false
		}
		eth::suicide(sender);
	}
}

impl RepoContract for RepoContractInstance {
	/// A contract constructor implementation.
	fn constructor(&mut self,
		borrower: Address,
		lender: Address,
		loan_token: Address,
		security_token: Address,
		loan_amount: U256,
		security_amount: U256,
		interest_rate: U256,
		activation_deadline: u64,
		return_deadline: u64
	) {
		self.storage.write(&BORROWER_KEY, &H256::from(borrower).into());
		self.storage.write(&LENDER_KEY, &H256::from(lender).into());
		self.storage.write(&LOAN_TOKEN_KEY, &H256::from(loan_token).into());
		self.storage.write(&SECURITY_TOKEN_KEY, &H256::from(security_token).into());
		self.storage.write(&LOAN_AMOUNT_KEY, &loan_amount.into());
		self.storage.write(&SECURITY_AMOUNT_KEY, &security_amount.into());
		self.storage.write(&INTEREST_RATE_KEY, &interest_rate.into());
		self.storage.write(&ACTIVATION_DEADLINE_KEY, &U256::from(activation_deadline).into());
		self.storage.write(&RETURN_DEADLINE_KEY, &U256::from(return_deadline).into());
	}

	// Tries to activate contract
	fn accept(&mut self) -> bool {
		let sender = eth::sender();
		if self.is_active() {
			panic!("Cannot accept, contract has activated already");
		}
		if eth::timestamp() > self.activation_deadline() {
			self.suicide(&sender);
		}
		let lender_address = self.lender_address();
		let borrower_address = self.borrower_address();

		// Accept by borrower
		if sender == borrower_address {
			self.storage.write(&BORROW_ACCEPTED_KEY, &U256::from(1).into());
			self.BorrowerAcceptance();
		}
		// Accept by lender
		else if sender == lender_address {
			self.storage.write(&LEND_ACCEPTED_KEY, &U256::from(1).into());
			self.LenderAcceptance();
		} else {
			panic!("Only for participants");
		}

		// Wait for all parties to accept
		if !(self.borrower_acceptance() && self.lender_acceptance()) {
			return false;
		}

		let mut loan_token = Token::new(self.loan_token_address());
		let mut security_token = Token::new(self.security_token_address());
		let loan_amount = self.loan_amount();
		let security_amount = self.security_amount();

		let this_contract_address = eth::address();
		// Transfer security from borrower_address to the contract address
		assert!(security_token.transferFrom(borrower_address, this_contract_address, security_amount));
		// Transfer loan to the borrower address
		assert!(loan_token.transferFrom(lender_address, borrower_address, loan_amount));

		return true;
	}

	fn terminate(&mut self) -> bool {
		let sender = eth::sender();
		if !self.is_active() && eth::timestamp() > self.activation_deadline() {
			self.suicide(&sender);
		}
		let lender_address = self.lender_address();
		let borrower_address = self.borrower_address();
		let mut loan_token = Token::new(self.loan_token_address());
		let mut security_token = Token::new(self.security_token_address());
		let loan_amount = self.loan_amount();
		let security_amount = self.security_amount();
		let interest_amount = (loan_amount / DIVISOR) * self.interest_rate();
		let return_amount = loan_amount + interest_amount;

		if eth::timestamp() <= self.return_deadline() {
			if sender != borrower_address {
				panic!("Only borrower can terminate contract if deadline hasn't came");
			}
			assert!(loan_token.transferFrom(borrower_address, lender_address, return_amount));
			assert!(security_token.transfer(borrower_address, security_amount));
			self.Refund();
			self.suicide(&sender)
		} else {
			assert!(security_token.transfer(lender_address, security_amount));
			self.Default();
			self.suicide(&sender)
		}
	}
}

#[cfg(test)]
#[macro_use]
extern crate pwasm_test;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
	extern crate std;
	extern crate pwasm_token_contract;

	use pwasm_test;
	use super::*;
	use self::pwasm_test::{ext_get, ext_update, ext_reset};
	use bigint::U256;
	use pwasm_std::hash::{Address, H160, H256};

	// Can't just alias Address for tuple struct initialization. Seems like a compiller bug
	static BORROWER_ADDR: Address = H160([
		0xea, 0x67, 0x4f, 0xdd, 0xe7, 0x14, 0xfd, 0x97, 0x9d, 0xe3,
		0xed, 0xf0, 0xf5, 0x6a, 0xa9, 0x71, 0x6b, 0x89, 0x8e, 0xc8]);
	static LENDER_ADDR: Address = H160([
		0xdb, 0x6f, 0xd4, 0x84, 0xcf, 0xa4, 0x6e, 0xee, 0xb7, 0x3c,
		0x71, 0xed, 0xee, 0x82, 0x3e, 0x48, 0x12, 0xf9, 0xe2, 0xe1
	]);
	static LOAN_TOKEN_ADDR: Address = H160([
		0x0f, 0x57, 0x2e, 0x52, 0x95, 0xc5, 0x7f, 0x15, 0x88, 0x6f,
		0x9b, 0x26, 0x3e, 0x2f, 0x6d, 0x2d, 0x6c, 0x7b, 0x5e, 0xc6
	]);
	static SECURITY_TOKEN_ADDR: Address = H160([
		0xcd, 0x17, 0x22, 0xf2, 0x94, 0x7d, 0xef, 0x4c, 0xf1, 0x44,
		0x67, 0x9d, 0xa3, 0x9c, 0x4c, 0x32, 0xbd, 0xc3, 0x56, 0x81
	]);
	static CONTRACT_ADDR: Address = H160([
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

	use self::pwasm_token_contract::{TokenContract, Endpoint as TokenEndpoint};

	#[derive(Default)]
	struct TokenMock {
		balanceOf: U256,
		totalSupply: U256,
		transfer: bool,
		approve: bool,
		allowance: U256,
		transferFrom: bool,
	}

	impl TokenMock {
		fn with_transfer(mut self, transfer: bool) -> TokenMock {
			self.transfer = transfer;
			self
		}
		fn with_transfer_from(mut self, transfer_from: bool) -> TokenMock {
			self.transferFrom = transfer_from;
			self
		}
	}

	impl TokenContract for TokenMock {
		fn constructor(&mut self, _total_supply: U256) {
		}
		fn balanceOf(&mut self, _owner: Address) -> U256 {
			self.balanceOf
		}
		fn totalSupply(&mut self) -> U256 {
			self.totalSupply
		}
		fn transfer(&mut self, _to: Address, _amount: U256) -> bool {
			self.transfer
		}
		fn approve(&mut self, _spender: Address, _value: U256) -> bool {
			self.approve
		}
		fn allowance(&mut self, _owner: Address, _spender: Address) -> U256 {
			self.allowance
		}
		fn transferFrom(&mut self, _from: Address, _to: Address, _amount: U256) -> bool {
			self.transferFrom
		}
	}

	fn default_contract() -> RepoContractInstance {
		let mut contract = RepoContractInstance::new();
		let loan_amount: U256 = 10000.into();
		let security_amount: U256 = 50000.into();
		let interest_rate: U256 = 3.into();
		let activation_deadline: u64 = 10;
		let return_deadline: u64 = 20;
		contract.constructor(BORROWER_ADDR.clone(),
			LENDER_ADDR.clone(),
			LOAN_TOKEN_ADDR.clone(),
			SECURITY_TOKEN_ADDR.clone(),
			loan_amount,
			security_amount,
			interest_rate,
			activation_deadline,
			return_deadline);
		contract
	}

	fn active_contract() -> RepoContractInstance {
		ext_reset(|e| e
			.sender(BORROWER_ADDR)
			.timestamp(5)
			.endpoint(LOAN_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(true)).into())
			.endpoint(SECURITY_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(true)).into())
		);
		let mut contract = default_contract();
		contract.accept();
		ext_update(|e| e.sender(LENDER_ADDR));
		contract.accept();
		contract
	}

	#[test]
	fn should_create_contract_with_storage () {
		ext_reset(|e| e);
		let mut contract = RepoContractInstance::new();
		let loan_amount: U256 = 10000.into();
		let security_amount: U256 = 50000.into();
		let interest_rate: U256 = 3.into();
		let activation_deadline: u64 = 10;
		let return_deadline: u64 = 20;

		contract.constructor(BORROWER_ADDR.clone(),
			LENDER_ADDR.clone(),
			LOAN_TOKEN_ADDR.clone(),
			SECURITY_TOKEN_ADDR.clone(),
			loan_amount,
			security_amount,
			interest_rate,
			activation_deadline,
			return_deadline);

		assert_eq!(contract.borrower_address(), BORROWER_ADDR);
		assert_eq!(contract.lender_address(), LENDER_ADDR);
		assert_eq!(contract.loan_token_address(), LOAN_TOKEN_ADDR);
		assert_eq!(contract.security_token_address(), SECURITY_TOKEN_ADDR);
		assert_eq!(contract.loan_amount(), loan_amount);
		assert_eq!(contract.security_amount(), security_amount);
		assert_eq!(contract.interest_rate(), interest_rate);
		assert_eq!(contract.activation_deadline(), activation_deadline);
		assert_eq!(contract.return_deadline(), return_deadline);
	}

	#[test]
	fn should_activate_contract () {
		ext_reset(|e| e
			.sender(BORROWER_ADDR)
			.timestamp(5)
			.endpoint(LOAN_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(true)).into())
			.endpoint(SECURITY_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(true)).into())
		);

		let mut contract = default_contract();
		assert_eq!(contract.accept(), false);
		assert_eq!(contract.borrower_acceptance(), true);
		// Set sender to lender
		ext_update(|e| e.sender(LENDER_ADDR));
		assert_eq!(contract.accept(), true);
		let ext_calls = ext_get().calls();
		assert_eq!(ext_calls.len(), 2, "2 transfer calls expected");
		let security_transfer = &ext_calls[0];
		let loan_transfer = &ext_calls[1];
		assert_eq!(security_transfer.address, SECURITY_TOKEN_ADDR);
		assert_eq!(loan_transfer.address, LOAN_TOKEN_ADDR);

		// Check transfers
		assert_eq!(Address::from(H256::from(&security_transfer.input[4..36])), BORROWER_ADDR);
		assert_eq!(Address::from(H256::from(&security_transfer.input[36..68])), CONTRACT_ADDR);
		assert_eq!(U256::from(H256::from(&security_transfer.input[68..100])), 50000.into());

		assert_eq!(Address::from(H256::from(&loan_transfer.input[4..36])), LENDER_ADDR);
		assert_eq!(Address::from(H256::from(&loan_transfer.input[36..68])), BORROWER_ADDR);
		assert_eq!(U256::from(H256::from(&loan_transfer.input[68..100])), 10000.into());

		assert_eq!(contract.lender_acceptance(), true);
		assert_eq!(contract.is_active(), true);
	}

	#[test]
	#[should_panic]
	fn should_panic_if_contract_cant_transfer_loan_token() {
		ext_reset(|e| e
			.sender(BORROWER_ADDR)
			.timestamp(5)
			.endpoint(LOAN_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(false)).into())
			.endpoint(SECURITY_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(true)).into())
		);

		let mut contract = default_contract();
		assert_eq!(contract.accept(), false);
		// Set sender to lender
		ext_reset(|e| e.sender(LENDER_ADDR));
		// Should panic because contact can't transfer amount of LOAN_TOKEN for some reason
		contract.accept();
	}

	#[test]
	#[should_panic]
	fn should_panic_if_contract_cant_transfer_security_token() {
		ext_reset(|e| e
			.sender(BORROWER_ADDR)
			.timestamp(5)
			.endpoint(LOAN_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(true)).into())
			.endpoint(SECURITY_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(false)).into())
		);

		let mut contract = default_contract();
		assert_eq!(contract.accept(), false);
		// Set sender to lender
		ext_update(|e| e.sender(LENDER_ADDR));
		// Should panic because contact can't transfer amount of LOAN_TOKEN for some reason
		contract.accept();
	}

	#[test]
	fn should_suicide_if_activation_deadline_came() {
		let mut contract = default_contract();
		ext_reset(|e| e.sender(BORROWER_ADDR).timestamp(11));
		assert_eq!(contract.accept(), false);
	}

	// Active contract tests
	#[test]
	#[should_panic]
	fn should_panic_if_terminate_before_accept() {
		let mut contract = default_contract();
		ext_reset(|e| e.sender(BORROWER_ADDR).timestamp(5));
		contract.terminate();
	}

	#[test]
	fn should_terminate_by_borrower() {
		let mut contract = active_contract();
		ext_reset(|e| e
			.sender(BORROWER_ADDR)
			.timestamp(15)
			.endpoint(LOAN_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer_from(true)).into())
			.endpoint(SECURITY_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer(true)).into())
		);

		contract.terminate();
		let ext_calls = ext_get().calls();
		assert_eq!(ext_calls.len(), 2, "2 transfer calls expected");

		let loan_transfer = &ext_calls[0];
		let security_transfer = &ext_calls[1];
		assert_eq!(security_transfer.address, SECURITY_TOKEN_ADDR);
		assert_eq!(loan_transfer.address, LOAN_TOKEN_ADDR);

		// Check transfers
		assert_eq!(Address::from(H256::from(&security_transfer.input[4..36])), BORROWER_ADDR);
		assert_eq!(U256::from(H256::from(&security_transfer.input[36..68])), 50000.into());

		assert_eq!(Address::from(H256::from(&loan_transfer.input[4..36])), BORROWER_ADDR);
		assert_eq!(Address::from(H256::from(&loan_transfer.input[36..68])), LENDER_ADDR);
		assert_eq!(U256::from(H256::from(&loan_transfer.input[68..100])), 10300.into()); // 10000 + 300 of interest
	}

	#[test]
	#[should_panic]
	fn should_not_be_able_to_terminate_by_anybody_exept_borrower_if_deadline_hasnt_came() {
		let mut contract = active_contract();
		ext_reset(|e| e.sender(LENDER_ADDR).timestamp(15));
		contract.terminate();
	}

	#[test]
	fn can_be_terminated_if_deadline() {
		let mut contract = active_contract();
		ext_reset(|e| e
			.sender(Address::new()) // by anybody
			.timestamp(25)
			.endpoint(SECURITY_TOKEN_ADDR, TokenEndpoint::new(TokenMock::default().with_transfer(true)).into())
		);

		contract.terminate();

		let ext_calls = ext_get().calls();
		assert_eq!(ext_calls.len(), 1, "1 transfer call expected");
		let security_transfer = &ext_calls[0];
		assert_eq!(security_transfer.address, SECURITY_TOKEN_ADDR);
		// Check transfers
		assert_eq!(Address::from(H256::from(&security_transfer.input[4..36])), LENDER_ADDR);
		assert_eq!(U256::from(H256::from(&security_transfer.input[36..68])), 50000.into());
	}
}
