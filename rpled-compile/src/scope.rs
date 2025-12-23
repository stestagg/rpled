
    struct Scope {
        variables: Vec<String>,
    }

    impl Scope {

        fn resolve_name(&self, name: &str) -> Option<usize> {
            self.variables.iter().position(|var| var == name)
        }

        fn allocate(&mut self, name: String) -> usize {
            if self.variables.contains(&name) {
                panic!("Variable '{}' already declared in this scope", name);
            }
            self.variables.push(name);
            self.variables.len() - 1
        }

    }