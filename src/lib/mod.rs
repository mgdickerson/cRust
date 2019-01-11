extern crate petgraph;
use petgraph::graph;

pub mod Lexer;
pub mod Parser;
pub mod Utility;
pub mod IR;
pub mod Graph;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}