use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Declared {
    pub vars: HashSet<String>,
    pub fns: HashMap<String, usize>,
}

impl Declared {
    pub fn new() -> Self {
        Declared {
            vars: HashSet::new(),
            fns: HashMap::new(),
        }
    }

    pub fn assert_undeclared(&self, name: &str) -> Result<(), String> {
        if self.vars.contains(name) || self.fns.contains_key(name) {
            Err(format!(
                "Variable or function name '{}' is already declared",
                name
            ))
        } else {
            Ok(())
        }
    }

    pub fn fn_def(&mut self, fn_name: &str, args: usize) -> Result<(), String> {
        self.assert_undeclared(fn_name)?;
        self.fns.insert(fn_name.to_string(), args);
        Ok(())
    }

    pub fn var_dec(&mut self, var_name: &str) -> Result<(), String> {
        self.assert_undeclared(var_name)?;
        self.vars.insert(var_name.to_string());
        Ok(())
    }

    pub fn var_undec(&mut self, var_name: &str) -> Result<(), String> {
        if self.vars.remove(var_name) {
            Ok(())
        } else {
            Err(format!(
                "Undeclaring variable '{}' that did not declare",
                &var_name
            ))
        }
    }

    pub fn novar(&self) -> Self {
        Declared {
            vars: HashSet::new(),
            fns: self.fns.clone(),
        }
    }
}
