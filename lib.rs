#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
extern crate alloc;

#[ink::contract(dynamic_storage_allocator = true)]
mod vote_manager {

    // use executor_trait::iexecutor::Executor;
    // use executor_trait::{Executor};
    use alloc::format;
    use alloc::vec::Vec;
    use alloc::string::String;

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
            Vec as StorageVec,
        },
        Box as StorageBox,
        traits::{
            PackedLayout,
            SpreadLayout,
        }
    };

    type VoteId = u64;
    type ChoiceId = u32;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    enum VoterState {
        Absent,
        Yea,
        Nay,
    }

    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Clone,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Choice {
        choice_id: ChoiceId,
        content: String,
        yea: u64,
    }

    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Vote {
        executed: bool,
        title: String,
        desc: String,
        start_date: u64,
        vote_time: u64,
        support_require_pct: u64,
        min_require_num: u64,
        support_num: u64,
        choices: StorageBox<StorageVec<Choice>>,
    }

    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct DisplayVote {
        executed: bool,
        title: String,
        desc: String,
        start_date: u64,
        vote_time: u64,
        support_require_pct: u64,
        min_require_num: u64,
        support_num: u64,
        choices: String,
    }


    #[ink(storage)]
    pub struct VoteManager {
        votes_length: u64,
        votes: StorageHashMap<VoteId, Vote>,
        voters: StorageHashMap<(VoteId, AccountId), ChoiceId>,
    }

    #[ink(event)]
    pub struct StartVote {
        #[ink(topic)]
        vote_id: VoteId,

        #[ink(topic)]
        creator: AccountId,
    }

    #[ink(event)]
    pub struct CastVote {
        #[ink(topic)]
        vote_id: VoteId,

        #[ink(topic)]
        voter: AccountId,

        support_choice: ChoiceId,
    }

    #[ink(event)]
    pub struct ExecuteVote {
        #[ink(topic)]
        vote_id: VoteId,
    }

    impl VoteManager {

        #[ink(constructor)]
        pub fn new() -> Self {
            Self { 
                votes_length: 0,
                votes: StorageHashMap::default(),
                voters: StorageHashMap::default(),
            }
        }

        #[ink(message)]
        pub fn new_vote(&mut self, title: String, desc: String, vote_time: u64, support_require_pct: u64, min_require_num: u64, choices: String) -> u64 {
            let vote_id = self.votes_length.clone();
            self.votes_length += 1;
            let start_date: u64 = self.env().block_timestamp();
            let mut vote = Vote{
                executed: false,
                title,
                desc,
                start_date: start_date,
                vote_time,
                support_require_pct,
                min_require_num,
                support_num: 0,
                choices: StorageBox::new(StorageVec::default()),
            };
            let mut index = 0;
            let split = choices.split(",");
            for choice_content in split {
                vote.choices.push(Choice{
                    choice_id: index,
                    content: String::from(choice_content),
                    yea: 0,
                });
                index += 1;
            }
            self.votes.insert(vote_id, vote);
            self.env().emit_event(StartVote{
                vote_id,
                creator: self.env().caller(),
            });
            vote_id
        }

        #[ink(message)]
        pub fn execute(&mut self, vote_id: VoteId) -> bool {
            if !self.vote_exists(vote_id) {
                return false;
            }
            true 
        }

        #[ink(message)]
        pub fn vote(&mut self, vote_id: VoteId, support_choice: u32, voter: AccountId) -> bool {
            if !self.vote_exists(vote_id) {
                return false;
            }
            if let Some(vote) = self.votes.get_mut(&vote_id) {
                if support_choice > vote.choices.len() {
                    return false;
                }
                // has voted
                if let Some(choice_id) = self.voters.get(&(vote_id, voter)) {
                    if *choice_id != support_choice {
                        let choices = &mut vote.choices;
                        choices.get_mut(*choice_id).unwrap().yea -= 1;
                        vote.support_num -= 1;
                    }
                } 
                let choices = &mut vote.choices;
                let voter_choice = choices.get_mut(support_choice).unwrap();
                voter_choice.yea += 1;
                self.voters.insert((vote_id, voter), support_choice);    
                vote.support_num += 1;
                self.env().emit_event(CastVote{
                    vote_id,
                    voter: self.env().caller(), 
                    support_choice,
                });
                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn query_all_vote(&mut self) -> alloc::vec::Vec<DisplayVote> {
            let mut v: alloc::vec::Vec<DisplayVote> = alloc::vec::Vec::new();
            for (_, val) in &self.votes {
                let vote = self.convert_vote_to_displayvote(&val);
                v.push(vote);
            }
            return v;
        }

        #[ink(message)]
        pub fn query_executed_vote(&mut self) -> alloc::vec::Vec<DisplayVote> {
            let mut v: alloc::vec::Vec<DisplayVote> = alloc::vec::Vec::new();
            for (_, val) in &self.votes {
                if self.is_vote_executed(&val) {
                    let vote = self.convert_vote_to_displayvote(&val);
                    v.push(vote);
                }
            }
            return v;
        }

        #[ink(message)]
        pub fn query_open_vote(&mut self) -> alloc::vec::Vec<DisplayVote> {
            let mut v: alloc::vec::Vec<DisplayVote> = alloc::vec::Vec::new();
            for (_, val) in &self.votes {
                if self.is_vote_open(&val) {
                    let vote = self.convert_vote_to_displayvote(&val);
                    v.push(vote);
                }
            }
            return v;
        }

        #[ink(message)]
        pub fn query_wait_vote(&mut self) -> alloc::vec::Vec<DisplayVote> {
            let mut v: alloc::vec::Vec<DisplayVote> = alloc::vec::Vec::new();
            for (_, val) in &self.votes {
                if self.is_vote_wait(&val) {
                    let vote = self.convert_vote_to_displayvote(&val);
                    v.push(vote);
                }
            }
            return v;
        }
 

        fn format_choices_to_string(&self, source: &StorageBox<StorageVec<Choice>>) -> String {
            let iter = source.iter();
                let mut choices_vec = Vec::new();
                for val in iter {
                    let s = format!("{0}:{1}", val.content.clone(), val.yea);
                    choices_vec.push(s);
                }
                let choices_content = choices_vec.join(","); 
                return choices_content;
        }

        fn convert_vote_to_displayvote(&self, vote: &Vote) -> DisplayVote {
            let choices = &vote.choices;
            let choices_content = self.format_choices_to_string(choices);
            let vote = DisplayVote{
                executed: vote.executed,
                title: vote.title.clone(),
                desc: vote.desc.clone(),
                start_date: vote.start_date,
                vote_time: vote.vote_time,
                support_require_pct: vote.support_require_pct,
                min_require_num: vote.min_require_num,
                support_num: vote.support_num,
                choices: choices_content,
            };
            vote
        }

        fn vote_exists(&self, vote_id: u64) -> bool {
            return vote_id < self.votes_length;
        }

        fn is_vote_open(&self, vote: &Vote) -> bool {
            return self.env().block_timestamp() < vote.start_date + vote.vote_time && !vote.executed;
        }

        fn is_vote_wait(&self, vote: &Vote) -> bool {
            return self.env().block_timestamp() > vote.start_date + vote.vote_time && !vote.executed;
        }

        fn is_vote_executed(&self, vote: &Vote) -> bool {
            return !vote.executed;
        }

        fn is_vote_finished(&self, vote: &Vote) -> bool {
            return self.env().block_timestamp() < vote.start_date + vote.vote_time;
        }
    }
}
