#![feature(specialization)]
#![allow(invalid_reference_casting)]
#[macro_use]
extern crate lazy_static;

pub mod components;
pub mod depend_graph;
pub mod font;
pub mod rhi;
pub mod renderer;
pub mod asset;
