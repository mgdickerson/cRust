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

    // spilled register
    gray,
}

impl Color {
    pub fn to_string(&self) -> String {
        match &self {
            // Comes out black
            Color::aqua => String::from("aqua"),
            // Comes out black
            Color::fushia => String::from("fushia"),

            // The rest are fine
            Color::brown => String::from("brown"),
            Color::red => String::from("red"),
            Color::purple => String::from("purple"),
            Color::orange => String::from("orange"),
            Color::green => String::from("green"),
            Color::blue => String::from("blue"),

            // spilled register
            Color::gray => String::from("gray"),
        }
    }
}

// This is entertaining
type Register1 = u32;

// TODO : Replace this with a usize value which maps the first 8 to colors and the rest are gray
#[derive(Clone, PartialEq, Debug)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
}

impl Register {
    pub fn to_string(&self) -> String {
        match &self {
            Register::R0 => String::from("R0"),
            Register::R1 => String::from("R1"),
            Register::R2 => String::from("R2"),
            Register::R3 => String::from("R3"),
            Register::R4 => String::from("R4"),
            Register::R5 => String::from("R5"),
            Register::R6 => String::from("R6"),
            Register::R7 => String::from("R7"),
            Register::R8 => String::from("R8"),
            Register::R9 => String::from("R9"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RegisterAllocation {
    reg: Register,
}

impl RegisterAllocation {
    pub fn allocate_register(reg_color: &Color) -> Self {
        match reg_color {
            Color::aqua => RegisterAllocation { reg: Register::R1 },
            Color::fushia => RegisterAllocation { reg: Register::R2 },
            Color::brown => RegisterAllocation { reg: Register::R3 },
            Color::red => RegisterAllocation { reg: Register::R4 },
            Color::purple => RegisterAllocation { reg: Register::R5 },
            Color::orange => RegisterAllocation { reg: Register::R6 },
            Color::green => RegisterAllocation { reg: Register::R7 },
            Color::blue => RegisterAllocation { reg: Register::R8 },
            Color::gray => RegisterAllocation { reg: Register::R9 },
        }
    }

    pub fn allocate_R0() -> Self {
        RegisterAllocation { reg: Register::R0 }
    }

    pub fn get_register(&self) -> Register {
        self.reg.clone()
    }
}
