use serde_json::Value;

impl fastn_jdebug::JDebug for fastn_unresolved::Import {
    fn debug(&self, source: &str) -> serde_json::Value {
        let mut o = serde_json::Map::new();

        let name = if self.module.package.0.is_empty() {
            self.module.name.0.to_string()
        } else {
            format!("{}/{}", self.module.package.0, self.module.name.0)
        };

        o.insert(
            "import".into(),
            match self.alias {
                Some(ref v) => format!("{name}=>{}", v.0),
                None => name,
            }
            .into(),
        );

        dbg!(&self);

        if let Some(ref v) = self.export {
            o.insert("export".into(), v.debug(source));
        }

        if let Some(ref v) = self.exposing {
            o.insert("exposing".into(), v.debug(source));
        }

        serde_json::Value::Object(o)
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::Export {
    fn debug(&self, source: &str) -> serde_json::Value {
        match self {
            fastn_unresolved::Export::All => "all".into(),
            fastn_unresolved::Export::Things(v) => {
                serde_json::Value::Array(v.iter().map(|v| v.debug(source)).collect())
            }
        }
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::AliasableIdentifier {
    fn debug(&self, _source: &str) -> serde_json::Value {
        match self.alias {
            Some(ref v) => format!("{}=>{}", self.name.0, v.0),
            None => self.name.0.to_string(),
        }
        .into()
    }
}

impl fastn_jdebug::JDebug for fastn_unresolved::ComponentInvocation {
    fn debug(&self, _source: &str) -> Value {
        todo!()
    }
}

impl<U: fastn_jdebug::JDebug, R: fastn_jdebug::JDebug> fastn_jdebug::JDebug
    for fastn_unresolved::UR<U, R>
{
    fn debug(&self, _source: &str) -> Value {
        todo!()
    }
}
