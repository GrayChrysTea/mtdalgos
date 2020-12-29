//! This module contains macro definitions for handling [`Result`]s and
//! [`Options`].

/// Destructures an [`Option`] and sends the item contained within back out
/// if the [`Option`] destructures to [`Some(..)`], else a [`std::io::Error`]
/// is propagated from the function.
#[macro_export]
macro_rules! unwrapoption {
    ($result: expr) => {
        match $result {
            Some(res) => res,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find requested item.",
                ))
            }
        }
    };
}

/// Unwraps a request to lock a [`std::sync::Mutex`] but propagates the error
/// as a [`std::io::Error`] instead of panicking.
#[macro_export]
macro_rules! unwrapmutex {
    ($result: expr) => {{
        match $result {
            Ok(lock) => lock,
            Err(_error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "The Mutex was poisoned.",
                ))
            }
        }
    }};
}

/// Unwraps a request to read a [`std::sync::mpsc::Receiver`] but propagates
/// the error as a [`std::io::Error`] instead of panicking.
#[macro_export]
macro_rules! unwrapreceiver {
    ($result: expr) => {{
        match $result {
            Ok(message) => message,
            Err(_error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "Transmitter was dropped.",
                ))
            }
        }
    }};
}

/// Unwraps a request to write to a [`std::sync::mpsc::Sender`] but propagates
/// the error as a [`std::io::Error`] instead of panicking.
#[macro_export]
macro_rules! unwrapsender {
    ($result: expr) => {{
        match $result {
            Ok(res) => res,
            Err(_error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "Receiver was dropped.",
                ))
            }
        }
    }};
}
