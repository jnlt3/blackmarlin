#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

/// Module for helper macros.
pub mod helpers;

/// The color module.
pub mod color;
/// The piece module
pub mod piece;
/// The square module.
pub mod square;
/// The file module.
pub mod file;
/// The rank module.
pub mod rank;
/// The bitboard module.
pub mod bitboard;
/// The castling module.
pub mod castling;
/// The sliders module.
pub mod sliders;
/// The chess_move module.
pub mod chess_move;

pub use color::*;
pub use piece::*;
pub use square::*;
pub use file::*;
pub use rank::*;
pub use bitboard::*;
pub use castling::*;
pub use sliders::*;
pub use chess_move::*;
