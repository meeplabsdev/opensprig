use core::fmt::{Debug, Display};

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {
        Err($crate::types::Error::from($crate::types::_anyhow!($($arg)*)))
    };
}

pub struct DebugError<T: Debug + Send + Sync + 'static>(pub T);
pub struct DisplayError<T: Display + Send + Sync + 'static>(pub T);
pub struct NonStdError<T: Send + Sync + 'static>(pub T);

impl<T> Debug for DebugError<T>
where
    T: Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Display for DebugError<T>
where
    T: Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Debug for DisplayError<T>
where
    T: Display + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> Display for DisplayError<T>
where
    T: Display + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

pub struct Error(anyhow::Error);
pub type Result<T, E = Error> = core::result::Result<T, E>;

impl Error {
    pub fn msg<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Self(anyhow::Error::msg(message))
    }

    pub fn context<C>(self, context: C) -> Self
    where
        C: Display + Send + Sync + 'static,
    {
        Self(self.0.context(context))
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl<T> From<DebugError<T>> for Error
where
    T: Debug + Send + Sync + 'static,
{
    fn from(error: DebugError<T>) -> Self {
        Self::msg(error)
    }
}

impl<T> From<DisplayError<T>> for Error
where
    T: Display + Send + Sync + 'static,
{
    fn from(error: DisplayError<T>) -> Self {
        Self::msg(error)
    }
}

impl<T> From<NonStdError<T>> for Error
where
    T: Send + Sync + 'static,
{
    fn from(_: NonStdError<T>) -> Self {
        Self::msg(core::any::type_name::<T>())
    }
}

// pub trait Context<T> {
//     fn context<C>(self, context: C) -> Result<T>
//     where
//         C: Display + Send + Sync + 'static;
// }

// impl<T> Context<T> for Option<T> {
//     fn context<C>(self, context: C) -> Result<T>
//     where
//         C: Display + Send + Sync + 'static,
//     {
//         match self {
//             Some(ok) => Ok(ok),
//             None => error!(context),
//         }
//     }
// }
