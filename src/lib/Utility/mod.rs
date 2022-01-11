//pub mod dlx;
pub mod display;
pub mod error;
pub mod source_file;
pub mod syntax_position;

pub trait Dummy {
    fn dummy() -> Self;
}
