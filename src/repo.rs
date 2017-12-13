// Conract doesn't need standard library and the `main` function.
// `cargo test` requires `std` and it provided the `main` which is why the "std" feature should be turned on for `cargo test`
#![cfg_attr(not(feature="std"), no_main)]
#![cfg_attr(not(feature="std"), no_std)]

extern crate pwasm_std;
extern crate pwasm_abi;
extern crate pwasm_repo_contract;

use pwasm_abi::eth::EndpointInterface;

/// The main function receives a pointer for the call descriptor.
#[no_mangle]
pub fn call(desc: *mut u8) {
    let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = pwasm_repo_contract::Endpoint::new(pwasm_repo_contract::RepoContractInstance::new());
    result.done(endpoint.dispatch(&args));
}

#[no_mangle]
pub fn deploy(desc: *mut u8) {
    let (args, _) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = pwasm_repo_contract::Endpoint::new(pwasm_repo_contract::RepoContractInstance::new());
    endpoint.dispatch_ctor(&args);
}
