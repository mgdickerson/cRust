use lib::Graph::node::{Node, NodeData, NodeId, NodeType};
use lib::IR::ir::{InstTy, Op, ValTy, Value};
use lib::IR::ir_manager::{BlockTracker, InstTracker};

use petgraph::graph::Graph;
use petgraph::prelude::NodeIndex;
use petgraph::visit::DfsPostOrder;

use super::{Rc, RefCell};
use lib::Graph::node::NodeType::entrance;
use petgraph::{Directed, Incoming, Outgoing};
use std::collections::HashMap;

#[derive(Clone)]
pub struct GraphManager {
    graph: Graph<Node, String, Directed, u32>,
    block_to_id_map: HashMap<usize, NodeIndex>,
    current_node_index: NodeIndex,
    main_node_index: NodeIndex,
    entrance_index: NodeIndex,
}

impl GraphManager {
    pub fn new(mut graph: Graph<Node, String, Directed, u32>, bt: &mut BlockTracker) -> Self {
        let entrance_node = Node::new(String::from("Entrance"), bt, NodeType::entrance);
        let current_node = Node::new(String::from("Main"), bt, NodeType::main_node);
        let entrance_node_index = graph.add_node(entrance_node);
        let current_node_index = graph.add_node(current_node);
        graph.add_edge(
            entrance_node_index,
            current_node_index,
            String::from("black"),
        );
        let main_node_index = current_node_index.clone();
        GraphManager {
            graph,
            block_to_id_map: HashMap::new(),
            current_node_index,
            main_node_index,
            entrance_index: entrance_node_index,
        }
    }

    // -- Node Related Functions -- //

    pub fn new_node(
        &mut self,
        node_tag: String,
        bt: &mut BlockTracker,
        node_type: NodeType,
    ) -> &mut NodeIndex {
        let current_node = Node::new(node_tag, bt, node_type);
        self.current_node_index = self.graph.add_node(current_node);
        self.get_mut_ref_current_node_index()
    }

    pub fn clone_graph(&self) -> Self {
        self.clone()
    }

    pub fn get_node_index(self) -> NodeIndex {
        self.current_node_index
    }

    pub fn clone_node_index(&self) -> NodeIndex {
        self.current_node_index.clone()
    }

    pub fn set_main_node(&mut self) {
        self.current_node_index = self.main_node_index.clone()
    }

    pub fn update_main_node(&mut self, node_id: NodeIndex) {
        self.main_node_index = node_id;
    }

    pub fn get_main_node(&self) -> NodeIndex {
        self.main_node_index.clone()
    }

    pub fn get_main_entrance_node(&self) -> NodeIndex {
        self.entrance_index.clone()
    }

    pub fn get_exit_nodes(&self, root_node: &NodeIndex) -> Vec<NodeIndex> {
        let search_path = self.graph_visitor(root_node.clone());
        let mut exit_ids = Vec::new();

        for node_id in search_path {
            if self.graph.node_weight(node_id).unwrap().get_node_type() == NodeType::exit {
                exit_ids.push(node_id);
            }
        }

        exit_ids
    }

    pub fn map_blocks_to_node_ids(&mut self) {
        let node_index_list = self.graph.node_indices();
        for node_id in node_index_list {
            let block_num = self.graph.node_weight(node_id).unwrap().get_node_id();

            self.block_to_id_map.insert(block_num, node_id);
        }
    }

    pub fn block_node_map(&self) -> &HashMap<usize, NodeIndex> {
        &self.block_to_id_map
    }

    pub fn get_current_id(&self) -> NodeIndex {
        self.current_node_index.clone()
    }

    pub fn get_mut_ref_current_node_index(&mut self) -> &mut NodeIndex {
        &mut self.current_node_index
    }

    pub fn switch_current_node_index(&mut self, new_node: NodeIndex) {
        self.current_node_index = new_node;
    }

    pub fn get_node_id(&self, node_index: NodeIndex) -> usize {
        self.graph
            .node_weight(node_index)
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
        self.graph
            .node_weight_mut(self.current_node_index)
            .expect("Expected Node to have weight, none was found while adding instruction.")
            .get_mut_data_ref()
            .add_instruction(Rc::clone(&inst_ref));
        Value::new(ValTy::op(Rc::clone(&inst_ref)))
    }

    pub fn add_instruction_in_node(&mut self, inst: Op, node_id: &NodeIndex) -> Value {
        let inst_ref = Rc::new(RefCell::new(inst));
        self.graph
            .node_weight_mut(node_id.clone())
            .expect("Expected Node to have weight when adding inst, found none.")
            .get_mut_data_ref()
            .add_instruction(Rc::clone(&inst_ref));
        Value::new(ValTy::op(Rc::clone(&inst_ref)))
    }

    pub fn insert_instruction(&mut self, position: usize, inst: Op) -> Value {
        let inst_ref = Rc::new(RefCell::new(inst));
        self.graph
            .node_weight_mut(self.current_node_index)
            .expect("Expected Node to have weight when inserting, found none.")
            .get_mut_data_ref()
            .insert_instruction(position, Rc::clone(&inst_ref));

        Value::new(ValTy::op(Rc::clone(&inst_ref)))
    }

    pub fn insert_instruction_in_node(
        &mut self,
        position: usize,
        inst: Op,
        node_id: &NodeIndex,
    ) -> Value {
        let inst_ref = Rc::new(RefCell::new(inst));
        self.graph
            .node_weight_mut(node_id.clone())
            .expect("Expected Node to have weight when inserting inst, found none.")
            .get_mut_data_ref()
            .insert_instruction(position, Rc::clone(&inst_ref));

        Value::new(ValTy::op(Rc::clone(&inst_ref)))
    }

    pub fn remove_inactive_inst(&mut self, inst_id: NodeIndex) {
        self.graph
            .node_weight_mut(inst_id)
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
        match self
            .graph
            .node_weight(node_id)
            .expect("Attempted to check non-existent node")
            .get_node_type()
        {
            NodeType::entrance | NodeType::exit => {
                return true;
            }
            _ => {
                // standard case, fall through
            }
        }

        let inst_vec = self
            .graph
            .node_weight_mut(node_id)
            .expect("Attempted to check non-existent node")
            .get_mut_data_ref()
            .get_mut_inst_list_ref();

        match inst_vec.len() {
            0 => false,
            1 => {
                let inst_ty = inst_vec[0].borrow().inst_type().clone();
                match inst_ty {
                    InstTy::bra => {
                        if let (_, Some(ValTy::node_id(node_id))) =
                            inst_vec[0].borrow().get_val_ty()
                        {
                            // fall through
                        } else if let (_, Some(ValTy::con(con))) = inst_vec[0].borrow().get_val_ty()
                        {
                            // fall through
                        } else {
                            return true;
                        }

                        inst_vec[0].borrow_mut().deactivate();
                        return false;
                    }
                    InstTy::blt
                    | InstTy::ble
                    | InstTy::bgt
                    | InstTy::bge
                    | InstTy::beq
                    | InstTy::bne => {
                        inst_vec[0].borrow_mut().deactivate();
                        false
                    }
                    _ => true,
                }
            }
            _ => true,
        }
    }
}
