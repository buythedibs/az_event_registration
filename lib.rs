#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod errors;

#[ink::contract]
mod az_event_registration {
    use crate::errors::AzEventRegistrationError;
    use ink::{reflect::ContractEventBase, storage::Mapping};

    // === TYPES ===
    type Event = <AzEventRegistration as ContractEventBase>::Type;
    type Result<T> = core::result::Result<T, AzEventRegistrationError>;

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
        address: AccountId,
        referrer: Option<AccountId>,
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

        #[ink(message)]
        pub fn show(&self, address: AccountId) -> Result<Registration> {
            self.registrations
                .get(address)
                .ok_or(AzEventRegistrationError::NotFound(
                    "Registration".to_string(),
                ))
        }

        // === HANDLES ===
        #[ink(message)]
        pub fn register(&mut self, referrer: Option<AccountId>) -> Result<Registration> {
            let caller: AccountId = Self::env().caller();
            if let Some(referrer_unwrapped) = referrer {
                if referrer_unwrapped == caller {
                    return Err(AzEventRegistrationError::UnprocessableEntity(
                        "Registrant and referrer must be different".to_string(),
                    ));
                }
            }
            if self.registrations.get(caller).is_some() {
                return Err(AzEventRegistrationError::UnprocessableEntity(
                    "Registration already exists".to_string(),
                ));
            }

            let registration = Registration {
                address: caller,
                referrer,
            };
            self.registrations.insert(caller, &registration);

            Ok(registration)
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
            let az_event_registration = AzEventRegistration::new();
            (accounts, az_event_registration)
        }

        // === TESTS ===
        // === TEST QUERIES ===
        #[ink::test]
        fn test_config() {
            let (accounts, az_event_registration) = init();
            let config = az_event_registration.config();
            // * it returns the config
            assert_eq!(config.admin, accounts.bob);
        }

        // === TEST HANDLES ===
        #[ink::test]
        fn test_register() {
            let (accounts, mut az_event_registration) = init();
            // when registration does not exist
            // = when referrer is present
            // == when referrer is different to caller
            // === * it create a new registration
            let mut referrer: Option<AccountId> = Some(accounts.alice);
            let mut result = az_event_registration.register(referrer);
            let mut result_unwrapped = result.unwrap();
            assert_eq!(
                result_unwrapped,
                Registration {
                    address: accounts.bob,
                    referrer
                }
            );
            // == when referrer is same as caller
            referrer = Some(accounts.bob);
            // === * it raises an error
            result = az_event_registration.register(referrer);
            assert_eq!(
                result,
                Err(AzEventRegistrationError::UnprocessableEntity(
                    "Registrant and referrer must be different".to_string()
                ))
            );
            // = when referrer is blank
            referrer = None;
            // = * it create a new registration
            set_caller::<DefaultEnvironment>(accounts.charlie);
            result = az_event_registration.register(referrer);
            result_unwrapped = result.unwrap();
            assert_eq!(
                result_unwrapped,
                Registration {
                    address: accounts.charlie,
                    referrer
                }
            );
            // when registration exists
            // * it raises an error
            result = az_event_registration.register(referrer);
            assert_eq!(
                result,
                Err(AzEventRegistrationError::UnprocessableEntity(
                    "Registration already exists".to_string()
                ))
            );
        }
    }
}
