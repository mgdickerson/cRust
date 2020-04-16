//pub mod dlx;
pub mod display;
pub mod syntax_position;
pub mod source_file;
pub mod error;

pub trait Dummy {
    fn dummy() -> Self;
}