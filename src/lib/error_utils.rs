
///
/// A glue between thiserror and anyhow libraries adding a functionality
/// similar to defining multiple #[from] tags.
/// 
/// Usage:
/// 
/// ```
/// use czkawka::from_error;
/// 
/// #[derive(Debug, thiserror::Error)]
/// pub enum DuzyError {
///     #[error("General Error")]
///     GeneralError(anyhow::Error),
/// 
///     #[error("Specific Error")]
///     SpecificError
/// }
/// 
/// from_error!(DuzyError::GeneralError, std::io::Error, std::num::ParseIntError);
/// 
/// fn fallo() -> Result<u32, DuzyError> {
/// 
///     std::fs::read_to_string("bad_file")?;          // io::std::Error
///     "a".parse::<u32>()?;                           // std::num::ParseIntError
///     Ok(Err(DuzyError::SpecificError)?)             // DuzyError::GuwnianyError
/// 
///     // ...
/// }
/// ```
///
#[macro_export]
macro_rules! from_error {
    ($error:ident::$variant:ident, $($source:ty),+ ) => {
        $(impl From<$source> for $error {
            fn from(value: $source) -> Self {
                $error::$variant(anyhow::Error::new(value))
            }
        })+
    };
}