mod alignment;
mod arithmetic;
mod breaker;
mod builder;
pub mod constants;
pub mod datastructures;
#[cfg(feature = "debug")]
mod debug;
mod dvi;
mod error;
mod extensions;
mod font_metric;
mod global;
pub mod initialization;
mod io;
mod math;
mod parser;
pub mod strings;

pub use global::Global;

// Types defined here
type ASCIICode = u8;
type HalfWord = i32;
type QuarterWord = u16;
pub type Integer = HalfWord;
type Real = f64;
type GlueRatio = Real;
type Scaled = Integer;
type StrNum = usize;
type SmallNumber = u16;
