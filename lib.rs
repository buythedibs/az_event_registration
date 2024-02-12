#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod az_event_registration {
    use ink::storage::Mapping;

    // === STRUCTS ===
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Config {
        admin: AccountId,
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Registration {
        id: u32,
        address: AccountId,
        referrer: AccountId,
    }

    // === CONTRACT ===
    #[ink(storage)]
    pub struct AzEventRegistration {
        admin: AccountId,
        registrations: Mapping<AccountId, Registration>,
    }
    impl AzEventRegistration {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                admin: Self::env().caller(),
                registrations: Mapping::default(),
            }
        }

        // === QUERIES ===
        #[ink(message)]
        pub fn config(&self) -> Config {
            Config { admin: self.admin }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{default_accounts, set_caller, DefaultAccounts},
            DefaultEnvironment,
        };

        // === HELPERS ===
        fn init() -> (DefaultAccounts<DefaultEnvironment>, AzEventRegistration) {
            let accounts = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.bob);
            let az_smart_contract_hub = AzEventRegistration::new();
            (accounts, az_smart_contract_hub)
        }

        // === TESTS ===
        // === TEST QUERIES ===
        #[ink::test]
        fn test_config() {
            let (accounts, az_smart_contract_hub) = init();
            let config = az_smart_contract_hub.config();
            // * it returns the config
            assert_eq!(config.admin, accounts.bob);
        }
    }
}
