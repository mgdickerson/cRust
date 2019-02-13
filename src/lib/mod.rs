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

/// General Structure to move towards:
///
/// // Keep track of Op results as temp variables.
/// struct TempVals {
///     Vec<Rc<RefCell<Value>>
/// }
///
/// // For the variable manager, switch to using Rc<RefCells> for
/// // the variables, so if one value changes it is reflected on all.
///
/// // Also need to implement the change on tracking usage. Build
/// // this in to building ops.
///
/// // Change the ops to have references to Rc<RefCells> so
/// // changes are reflected. Might also want to rethink how
/// // the current p_command works...
///
/// // Build way for variable manager to update values assigned
/// // with old values.
///

pub fn test() {

}