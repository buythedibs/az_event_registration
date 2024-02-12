use ink::{
    env::Error as InkEnvError,
    prelude::{format, string::String},
    LangError,
};
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum AzEventRegistrationError {
    ContractCall(LangError),
    InkEnvError(String),
    NotFound(String),
    Unauthorised,
    UnprocessableEntity(String),
}
impl From<InkEnvError> for AzEventRegistrationError {
    fn from(e: InkEnvError) -> Self {
        AzEventRegistrationError::InkEnvError(format!("{e:?}"))
    }
}
impl From<LangError> for AzEventRegistrationError {
    fn from(e: LangError) -> Self {
        AzEventRegistrationError::ContractCall(e)
    }
}
