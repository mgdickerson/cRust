use std::collections::HashMap;

pub struct DefChain {
    variables: HashMap<String, usize>,
}

impl DefChain {
    pub fn new() -> Self {
        DefChain { variables: HashMap::new() }
    }

    /*
    pub fn get_unique_tag(&mut self, var: & String) -> String {
        match self.variables.get_mut(var) {
            Some(T) => {
                *T += 1;
                let unique_tag = String::from("%") + var + "_" + &T.clone().to_string();
                self.variables.insert(unique_tag.clone(), 0);
                unique_tag
            },
            None => {
                self.variables.insert(var.clone(), 0);
                let unique_tag = String::from("%") + var + "_0";
                unique_tag
            }
        }
    }
    */
}