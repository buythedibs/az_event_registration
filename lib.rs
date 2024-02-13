#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod errors;

#[ink::contract]
mod az_event_registration {
    use crate::errors::AzEventRegistrationError;
    use ink::{codegen::EmitEvent, reflect::ContractEventBase, storage::Mapping};

    // === TYPES ===
    type Event = <AzEventRegistration as ContractEventBase>::Type;
    type Result<T> = core::result::Result<T, AzEventRegistrationError>;

    // === EVENTS ===
    #[ink(event)]
    pub struct Register {
        #[ink(topic)]
        address: AccountId,
        #[ink(topic)]
        referrer: Option<AccountId>,
    }

    #[ink(event)]
    pub struct Update {
        #[ink(topic)]
        address: AccountId,
        #[ink(topic)]
        referrer: Option<AccountId>,
    }

    // === STRUCTS ===
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Config {
        admin: AccountId,
        deadline: Timestamp,
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
        deadline: Timestamp,
        registrations: Mapping<AccountId, Registration>,
    }
    impl AzEventRegistration {
        #[ink(constructor)]
        pub fn new(deadline: Timestamp) -> Self {
            Self {
                admin: Self::env().caller(),
                deadline,
                registrations: Mapping::default(),
            }
        }

        // === QUERIES ===
        // deadline is u64
        // Javascript works with this number new Date(1707789561000)
        #[ink(message)]
        pub fn config(&self) -> Config {
            Config {
                admin: self.admin,
                deadline: self.deadline,
            }
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
        pub fn destroy(&mut self) -> Result<()> {
            let caller: AccountId = Self::env().caller();
            self.show(caller)?;

            self.registrations.remove(caller);

            Ok(())
        }

        #[ink(message)]
        pub fn register(&mut self, referrer: Option<AccountId>) -> Result<Registration> {
            let caller: AccountId = Self::env().caller();
            let block_timestamp: Timestamp = Self::env().block_timestamp();
            if block_timestamp > self.deadline {
                return Err(AzEventRegistrationError::UnprocessableEntity(
                    "Registration is now closed".to_string(),
                ));
            }
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

            Self::emit_event(
                self.env(),
                Event::Register(Register {
                    address: caller,
                    referrer,
                }),
            );

            Ok(registration)
        }

        #[ink(message)]
        pub fn update(&mut self, referrer: Option<AccountId>) -> Result<Registration> {
            let caller: AccountId = Self::env().caller();
            let mut registration: Registration = self.show(caller)?;
            registration.referrer = referrer;
            self.registrations.insert(caller, &registration);

            Self::emit_event(
                self.env(),
                Event::Update(Update {
                    address: caller,
                    referrer,
                }),
            );

            Ok(registration)
        }

        #[ink(message)]
        pub fn update_config(&mut self, deadline: Timestamp) -> Result<()> {
            let caller: AccountId = Self::env().caller();
            Self::authorise(caller, self.admin)?;

            self.deadline = deadline;

            Ok(())
        }

        fn authorise(allowed: AccountId, received: AccountId) -> Result<()> {
            if allowed != received {
                return Err(AzEventRegistrationError::Unauthorised);
            }

            Ok(())
        }

        fn emit_event<EE: EmitEvent<Self>>(emitter: EE, event: Event) {
            emitter.emit_event(event);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{default_accounts, set_caller, DefaultAccounts},
            DefaultEnvironment,
        };

        const MOCK_DEAD_LINE: Timestamp = 654654;

        // === HELPERS ===
        fn init() -> (DefaultAccounts<DefaultEnvironment>, AzEventRegistration) {
            let accounts = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.bob);
            let az_event_registration = AzEventRegistration::new(MOCK_DEAD_LINE);
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
            assert_eq!(config.deadline, MOCK_DEAD_LINE)
        }

        // === TEST HANDLES ===
        #[ink::test]
        fn test_destroy() {
            let (accounts, mut az_event_registration) = init();
            let referrer: Option<AccountId> = None;
            // when registration does not exist
            // * it raises an error
            let mut result = az_event_registration.update(referrer);
            assert_eq!(
                result,
                Err(AzEventRegistrationError::NotFound(
                    "Registration".to_string()
                ))
            );
            // when registration exists
            result = az_event_registration.register(referrer);
            result.unwrap();
            // * it destroys the registration
            az_event_registration.destroy().unwrap();
            result = az_event_registration.show(accounts.bob);
            assert_eq!(
                result,
                Err(AzEventRegistrationError::NotFound(
                    "Registration".to_string()
                ))
            );
        }

        #[ink::test]
        fn test_register() {
            let (accounts, mut az_event_registration) = init();
            let mut referrer: Option<AccountId> = Some(accounts.alice);
            // when current block timestamp is greater than deadline
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(
                az_event_registration.deadline + 1,
            );
            let mut result = az_event_registration.register(referrer);
            assert_eq!(
                result,
                Err(AzEventRegistrationError::UnprocessableEntity(
                    "Registration is now closed".to_string()
                ))
            );
            // when current block timestamp is less than or equal to deadline
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(
                az_event_registration.deadline,
            );
            // = when registration does not exist
            // == when referrer is present
            // === when referrer is different to caller
            // ==== * it create a new registration
            result = az_event_registration.register(referrer);
            let mut result_unwrapped = result.unwrap();
            assert_eq!(
                result_unwrapped,
                Registration {
                    address: accounts.bob,
                    referrer
                }
            );
            // === when referrer is same as caller
            referrer = Some(accounts.bob);
            // ==== * it raises an error
            result = az_event_registration.register(referrer);
            assert_eq!(
                result,
                Err(AzEventRegistrationError::UnprocessableEntity(
                    "Registrant and referrer must be different".to_string()
                ))
            );
            // == when referrer is blank
            referrer = None;
            // == * it create a new registration
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
            // = when registration exists
            // = * it raises an error
            result = az_event_registration.register(referrer);
            assert_eq!(
                result,
                Err(AzEventRegistrationError::UnprocessableEntity(
                    "Registration already exists".to_string()
                ))
            );
        }

        #[ink::test]
        fn test_update() {
            let (accounts, mut az_event_registration) = init();
            let mut referrer: Option<AccountId> = None;
            // when registration does not exist
            // * it raises an error
            let mut result = az_event_registration.update(referrer);
            assert_eq!(
                result,
                Err(AzEventRegistrationError::NotFound(
                    "Registration".to_string()
                ))
            );
            // when registration exists
            result = az_event_registration.register(referrer);
            result.unwrap();
            // = when registrater does not have a reffer
            // == when adding a new referrer
            // == * it updates the referrer
            referrer = Some(accounts.charlie);
            result = az_event_registration.update(referrer);
            let mut result_unwrapped = result.unwrap();
            assert_eq!(
                result_unwrapped,
                Registration {
                    address: accounts.bob,
                    referrer
                }
            );
            // = when registrater has a reffer
            // == when removing the referrer
            // == * it updates the referrer
            referrer = None;
            result = az_event_registration.update(referrer);
            result_unwrapped = result.unwrap();
            assert_eq!(
                result_unwrapped,
                Registration {
                    address: accounts.bob,
                    referrer
                }
            );
        }

        #[ink::test]
        fn test_update_config() {
            let (accounts, mut az_event_registration) = init();
            let new_deadline: Timestamp = 123456789;
            // when called by non-admin
            set_caller::<DefaultEnvironment>(accounts.charlie);
            // * it raises an error
            let result = az_event_registration.update_config(new_deadline);
            assert_eq!(result, Err(AzEventRegistrationError::Unauthorised));
            // when called by admin
            set_caller::<DefaultEnvironment>(accounts.bob);
            // * it updates the config
            az_event_registration.update_config(new_deadline).unwrap();
            assert_eq!(az_event_registration.deadline, new_deadline);
        }
    }
}
