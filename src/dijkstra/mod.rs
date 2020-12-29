//! Module for calculating the shortest distances to go to each node in a
//! graph from one starting node if possible.
//! 
//! This module has 2 editions: [`self::complex`] and [`self::simple`].
//! As of writing, [`self::simple`] is somewhat complete while
//! [`self::complex`] is not ready. The main difference between the 2
//! editions is that you can use generic identifiers and numbers to calculate
//! costs in the [`self::complex`] edition but you can only use [`usize`] and
//! [`u128`] for identifiers and costs respectively in the [`self::simple`]
//! edition.

pub mod complex;
pub mod simple;
