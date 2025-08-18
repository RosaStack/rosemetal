pub mod abbrev;
pub mod bitcodes;
pub mod bitcursor;
pub mod bitstream;
pub mod parser;
pub mod record;

pub use abbrev::*;
pub use bitcodes::*;
pub use bitcursor::*;
pub use bitstream::*;
pub use parser::*;
pub use record::*;

#[allow(unused_variables)]
pub fn debug(content: &str) {
    #[cfg(feature = "debug")]
    println!("{:?}", content);
}
