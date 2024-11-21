pub struct TDoc<'a> {
    pub name: &'a str,
    pub definitions: &'a indexmap::IndexMap<String, fastn_resolved::Definition>,
    pub builtins: &'a indexmap::IndexMap<String, fastn_resolved::Definition>,
}

impl TDoc<'_> {
    fn get(&self, name: &str) -> Option<&fastn_resolved::Definition> {
        self.definitions
            .get(name)
            .or_else(|| self.builtins.get(name))
    }
}

impl<'a> fastn_resolved::tdoc::TDoc for TDoc<'a> {
    fn get_opt_function(&self, name: &str) -> Option<fastn_resolved::Function> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Function(f)) => Some(f.clone()),
            _ => None,
        }
    }

    fn get_opt_record(&self, name: &str) -> Option<fastn_resolved::Record> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Record(f)) => Some(f.clone()),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        self.name
    }

    fn get_opt_component(&self, name: &str) -> Option<fastn_resolved::ComponentDefinition> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Component(f)) => Some(f.clone()),
            _ => None,
        }
    }

    fn get_opt_web_component(&self, name: &str) -> Option<fastn_resolved::WebComponentDefinition> {
        match self.get(name) {
            Some(fastn_resolved::Definition::WebComponent(f)) => Some(f.clone()),
            _ => None,
        }
    }
}
