use lib::IR::variable_manager::{VariableManager, UniqueVariable};

pub struct FunctionManager {
    func_name: String,

    // Functions will have their own Variable Managers
    // just to make things easy
    variable_manager: VariableManager,

    // Keeping a list of affected variables will
    // allow for only storing global vars when called
    affected_globals: Vec<String>,
}

impl FunctionManager {

}