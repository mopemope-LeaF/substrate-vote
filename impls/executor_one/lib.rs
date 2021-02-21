#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

// pub use self::executor_one::{IExecutor, Executor};

#[ink::trait_definition]
pub trait IExecutor {

    #[ink(constructor)]
    fn new() -> Self;

    #[ink(message)]
    fn execute(&mut self);
}

#[ink::contract]
mod executor_one {
    
    use super::IExecutor;

    #[ink(event)]
    pub struct Event {
        
        #[ink(topic)]
        name: String,

        #[ink(topic)]
        caller: AccountId,
    }

    #[ink(storage)]
    pub struct Executor {
        name: String,
    }

    impl IExecutor for Executor {
        #[ink(constructor)]
        fn new () -> Self {
           Self {
               name: "executor one"
           } 
        }

        #[ink(message)]
        fn execute(&mut self) {
            self.env().emit_event(Event{
                name: self.name,
                caller: self.env().caller(),
            });
        }
    }
}