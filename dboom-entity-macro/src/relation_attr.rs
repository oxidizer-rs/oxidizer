use darling::FromMeta;

#[derive(Debug,FromMeta)]
pub struct RelationAttr {
    pub model: String,
    pub key: String,
}