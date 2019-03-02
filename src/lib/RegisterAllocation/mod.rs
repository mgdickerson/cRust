pub mod interference_graph;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use lib::Graph::node::Node;

use super::{petgraph,Graph};
use petgraph::prelude::NodeIndex;
use petgraph::Directed;