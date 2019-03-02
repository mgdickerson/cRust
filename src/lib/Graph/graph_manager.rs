use lib::IR::ir::{Value,ValTy,Op,InstTy};
use lib::Graph::node::{Node, NodeId, NodeType, NodeData};
use lib::IR::ir_manager::{InstTracker, BlockTracker};

use petgraph::graph::Graph;
use petgraph::prelude::NodeIndex;
use petgraph::visit::DfsPostOrder;

use super::{Rc,RefCell};
use petgraph::{Outgoing,Incoming, Directed};

#[derive(Clone)]
pub struct GraphManager {
    graph: Graph<Node, String, Directed, u32>,
    current_node_index: NodeIndex,
    main_node_index: NodeIndex,
}

impl GraphManager {
    pub fn new(mut graph: Graph<Node, String, Directed, u32>, it: &mut InstTracker, bt: &mut BlockTracker) -> Self {
        let current_node = Node::new(String::from("Main_Node"), it, bt, NodeType::main_node);
        let current_node_index = graph.add_node(current_node);
        let main_node_index = current_node_index.clone();
        GraphManager { graph, current_node_index, main_node_index }
    }

    // -- Node Related Functions -- //

    pub fn new_node(&mut self, node_tag: String, it: &mut InstTracker, bt: &mut BlockTracker, node_type: NodeType) -> &mut NodeIndex {
        let current_node = Node::new(node_tag, it, bt, node_type);
        self.current_node_index = self.graph.add_node(current_node);
        self.get_mut_ref_current_node_index()
    }

    pub fn clone_graph(&self) -> Self {
        self.clone()
    }

    pub fn get_node_index(self) -> NodeIndex {
        self.current_node_index
    }

    pub fn clone_node_index(&self) -> NodeIndex { self.current_node_index.clone() }

    pub fn set_main_node(&mut self) { self.current_node_index = self.main_node_index.clone() }

    pub fn update_main_node(&mut self, node_id: NodeIndex) {
        self.main_node_index = node_id;
    }

    pub fn get_main_node(&self) -> NodeIndex {
        self.main_node_index.clone()
    }

    pub fn get_mut_ref_current_node_index(&mut self) -> &mut NodeIndex {
        &mut self.current_node_index
    }

    pub fn switch_current_node_index(&mut self, new_node: NodeIndex) {
        self.current_node_index = new_node;
    }

    pub fn get_node_id(&self, node_index: NodeIndex) -> usize {
        self.graph.node_weight(node_index)
            .expect("Expected node weight for node_id")
            .get_node_id()
    }

    pub fn add_edge(&mut self, parent: NodeIndex, child: NodeIndex) {
        self.graph.add_edge(parent, child, String::from("black"));
    }

    pub fn update_edge(&mut self, parent: NodeIndex, child: NodeIndex) {
        self.graph.add_edge(parent, child, String::from("black"));
    }

    pub fn add_dominance_edge(&mut self, parent: NodeIndex, child: NodeIndex) {
        self.graph.add_edge(parent, child, String::from("red"));
    }

    pub fn add_temp_dominance_edge(&mut self, parent: NodeIndex, child: NodeIndex, color: String) {
        self.graph.add_edge(parent, child, color);
    }

    // -- Graph Related Functions -- //

    pub fn get_mut_ref_graph(&mut self) -> &mut Graph<Node, String, Directed, u32> {
        &mut self.graph
    }

    pub fn get_ref_graph(&self) -> &Graph<Node, String, Directed, u32> {
        &self.graph
    }

    pub fn get_graph(self) -> Graph<Node, String, Directed, u32> {
        self.graph
    }

    // -- Convenience Feature for adding inst -- //

    pub fn add_instruction(&mut self, inst: Op) -> Value {
        let inst_ref = Rc::new(RefCell::new(inst));
        self.graph.node_weight_mut(self.current_node_index)
            .expect("Expected Node to have weight, none was found while adding instruction.")
            .get_mut_data_ref()
            .add_instruction(Rc::clone(&inst_ref));
        Value::new(ValTy::op(Rc::clone(&inst_ref)))
    }

    pub fn insert_instruction(&mut self, position: usize, inst: Op) -> Value {
        let inst_ref = Rc::new(RefCell::new(inst));
        self.graph.node_weight_mut(self.current_node_index)
            .expect("Expected Node to have weight when inserting, found none.")
            .get_mut_data_ref()
            .insert_instruction(position, Rc::clone(&inst_ref));

        Value::new(ValTy::op(Rc::clone(&inst_ref)))
    }

    pub fn remove_inactive_inst(&mut self, inst_id: NodeIndex) {
        self.graph.node_weight_mut(inst_id)
            .expect("Node does not exist or has been removed, cannot remove inactive instructions.")
            .get_mut_data_ref()
            .remove_inactive_inst();
    }

    pub fn graph_visitor(&self, root_node: NodeIndex) -> Vec<NodeIndex> {
        let mut dfs_post_order = DfsPostOrder::new(self.get_ref_graph(), root_node);
        let mut graph_visitor = Vec::new();

        // This gets a pretty good order of nodes to visit, though for some reason it still goes down the right path first.
        while let Some(node_id) = dfs_post_order.next(self.get_ref_graph()) {
            graph_visitor.push(node_id);
        }

        graph_visitor.reverse();
        graph_visitor
    }

    /// Function for checking that a node contains any instructions
    /// (or if it only contains a branch instruction). Returns
    /// true if node is valid, false if it is not and needs to be
    /// removed.
    pub fn check_node(&mut self, node_id: NodeIndex) -> bool {
        let inst_vec = self.graph.node_weight_mut(node_id)
            .expect("Attempted to check non-existent node").get_mut_data_ref()
            .get_mut_inst_list_ref();

        match inst_vec.len() {
            0 => {
                false
            },
            1 => {
                let inst_ty = inst_vec[0].borrow().inst_type().clone();
                match inst_ty {
                    InstTy::bra => {
                        if let (_, Some(ValTy::node_id(node_id))) = inst_vec[0].borrow().get_val_ty() {
                            // fall through
                        } else if let (_, Some(ValTy::con(con))) = inst_vec[0].borrow().get_val_ty() {
                            // fall through
                        } else {
                            return true
                        }

                        inst_vec[0].borrow_mut().deactivate();
                        return false
                    },
                    InstTy::blt | InstTy::ble |
                    InstTy::bgt | InstTy::bge | InstTy::beq |
                    InstTy::bne => {
                        inst_vec[0].borrow_mut().deactivate();
                        false
                    },
                    _ => {
                        true
                    },
                }
            },
            _ => {
                true
            },
        }
    }
}