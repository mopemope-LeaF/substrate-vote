#![cfg_attr(not(feature = "std"), no_std)]

// pub use self::iexecutor::IExecutor;
use ink_lang as ink;

// #[ink::contract]
// pub mod iexecutor {
#[ink::trait_definition]
pub trait IExecutor {

    #[ink(constructor)]
    fn new() -> Self;

    #[ink(message)]
    fn execute(&mut self);
}

//     #[ink(storage)]
//     pub struct Executor{}

//     impl Executor {
//         #[ink(constructor)]
//         pub fn new() -> Self {
//             Self {}
//         }
//     }
// }