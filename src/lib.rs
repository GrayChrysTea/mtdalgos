//! This library crate is a collection of algorithms reimplemented using
//! multithreading with help from the Rust standard library.
//! 
//! A thread pool is used to control the number of threads which can be used
//! by an algorithm and functions are passed to the workers in the thread pool
//! as a closure. The resulting object is directly edited by each worker
//! upon completion of their job through a mutex and said object can be
//! accessed upon the completion of all jobs.
//! 
//! These are the algorithms implemented so far:
//! 1. [`crate::dijkstra`].

pub mod dijkstra;
pub mod macros;
pub mod pool;
