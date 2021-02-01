#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod vote {

    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{
            PackedLayout,
            SpreadLayout,
        }
    };

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    enum VoterState {
        Absent,
        Yea,
        Nay,
    }

    type VoterStateMap = StorageHashMap<AccountId, VoterState>;

    #[ink(storage)]
    pub struct Vote {
        executed: bool,
        start_date: u64,
        vote_time: u64,
        support_require_pct: u64,
        yea: u64,
        nay: u64,
        voters: VoterStateMap,
    }

    impl Vote {

        #[ink(constructor)]
        pub fn new(_vote_time: u64, _support_require_pct: u64) -> Self {
            Self { 
                executed: false,
                start_date: Self::env().block_timestamp(),
                support_require_pct: _support_require_pct,
                vote_time: _vote_time,
                yea: 0,
                nay: 0,
                voters: StorageHashMap::default(),
            }
        }

        #[ink(message)]
        pub fn vote(&mut self, _supports: bool, _voter: AccountId) {
            let vote_state = self.voters.get(&_voter);
            if vote_state.is_some() {
                match vote_state {
                    None => (),
                    Some(VoterState::Yea) => self.yea -= 1,
                    Some(VoterState::Nay) => self.nay -= 1,
                    Some(VoterState::Absent) => (),
                }
            }
            if _supports {
                self.yea += 1;
                self.voters[&_voter] = VoterState::Yea;
            } else {
                self.nay += 1;
                self.voters[&_voter] = VoterState::Nay;
            }
        }
        
    }
}
