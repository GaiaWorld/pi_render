#![feature(specialization)]
#![allow(invalid_reference_casting)]
#![allow(irrefutable_let_patterns)]
#[macro_use]
extern crate lazy_static;

pub mod components;
pub mod depend_graph;
pub mod font;
pub mod rhi;
pub mod renderer;
pub mod asset;
