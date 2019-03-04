pub mod interference_graph;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lib::Graph::node::Node;

use super::{petgraph, Graph};
use petgraph::prelude::NodeIndex;
use petgraph::Directed;

pub enum Color {
    aqua,
    fushia,
    brown,
    red,
    purple,
    orange,
    green,
    blue,
}
