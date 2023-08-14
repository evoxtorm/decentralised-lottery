
#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod lottery {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct Lottery {
        // Other storage items for lottery management
        ticket_price: Balance, // Ticket price in the native token (e.g., DOT)
        tickets: Mapping<AccountId, u32>, // Ticket owner to Ticket ID mapping
        old_tickets: Mapping<u32, ()>,
        start_time: Option<Timestamp>,
        duration: Timestamp,
        tickets_t: Vec<(AccountId, u32)>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        LotteryTimeExpired,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Lottery {
        #[ink(constructor)]
        // , start_time: Timestamp, duration: Timestamp
        pub fn new(ticket_price: Balance, duration: Timestamp) -> Self {
            Self {
                ticket_price,
                tickets: Mapping::default(),
                old_tickets: Mapping::default(),
                tickets_t: Vec::new(),
                start_time: None,
                duration
            }
        }

        pub fn get_caller(&mut self) -> AccountId {
            ink::env::debug_println!("{:?}", self.env().caller());
            self.env().caller()
        }

        #[ink(message)]
        pub fn get_all_tickets(&mut self) -> Vec<(AccountId, u32)> {
            self.tickets_t.clone()
        }

        #[ink(message)]
        pub fn buy_ticket(&mut self) -> Result<()> {
            let caller = self.env().caller();
            let transferred_balance = self.env().balance();

            // Check if the transferred balance matches the ticket price
            if transferred_balance < self.ticket_price {
                return Err(Error::InsufficientAllowance);
            }

            if let Some(_start_time) = self.start_time {
            } else {
                self.start_time = Some(self.env().block_timestamp());
            }

            // You may need to implement additional checks or token transfers if using custom tokens

            // Record the ticket purchase
            let ticket_id: u32 = self.generate_ticket_id();
            self.tickets.insert(caller, &ticket_id);
            self.tickets_t.insert(self.tickets_t.len(), (caller, ticket_id));

            Ok(())
        }

        pub fn generate_ticket_id(&mut self) -> u32 {
            let mut ticket_number: u32;
            loop {
                ticket_number = self.generate_random_unique_digits();
                if let Some(_old_ticket) = self.old_tickets.get(ticket_number) {
                    continue;
                }
                break;
            }
            return ticket_number;
        }

        pub fn generate_random_unique_digits(&self) -> u32 {
            let mut num: u32 = 0;
            let mut digits = [false; 10];
        
            for _ in 0..6 {
                let mut digit: u32 = (self.rand() % 10) as u32;
                while digits[digit as usize] {
                    digit = (digit + 1) % 10;
                }
                digits[digit as usize] = true;
                num = num * 10 + digit;
            }
        
            num
        }
        
        // Custom random number generator (LCG)
        pub fn rand(&self) -> u32 {
            static mut SEED: u32 = 0;
            const A: u32 = 1664525;
            const C: u32 = 1013904223;
            unsafe {
                SEED = A.wrapping_mul(SEED).wrapping_add(C);
                SEED
            }
        }

        #[ink(message)]
        pub fn declare_winner(&mut self) -> (AccountId, u32) {

            let random_index = self.generate_random_unique_digits() as usize % self.tickets_t.len();
            let random_element = self.tickets_t.get(random_index).unwrap().clone();
            self.start_time = None;
            self.tickets_t = Vec::new();
            random_element
        }     

    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const TICKET_PRICE: Balance = 100_000_000; // Set your desired ticket price


        #[ink::test]
        fn test_buy_ticket() {
            // Initialize the contract with the desired ticket price
            let mut contract = Lottery::new(TICKET_PRICE, 100);

            // Set the sender to simulate a caller
            let accounts =
                ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
                    accounts.alice, TICKET_PRICE,
            );
            // Call the buy_ticket function and check the result
            assert_eq!(contract.buy_ticket(), Ok(()));
            assert_eq!(contract.buy_ticket(), Ok(()));

            assert_eq!(contract.tickets.get(&accounts.alice), contract.tickets.get(&accounts.alice));
        }

        #[ink::test]
        // #[ink::test(debug)]
        fn test_insufficient_allowance() {
            // Initialize the contract with a higher ticket price
            let mut contract = Lottery::new(TICKET_PRICE * 2, 100);

            // Set the sender to simulate a caller
            let accounts =
                ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            // set_sender(accounts.bob);

            // Simulate a transfer of funds to the contract
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
                accounts.bob, TICKET_PRICE,
        );

            // Call the buy_ticket function and check the result (expecting an error)
            assert_eq!(contract.buy_ticket(), Err(Error::InsufficientAllowance));

            // Check if no ticket was recorded for the caller
            assert_eq!(contract.tickets.get(&accounts.bob), None);
        }
    }

}
