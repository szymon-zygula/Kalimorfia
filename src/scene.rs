use crate::entities::entity::SceneEntity;

pub struct SceneEntityEntry<'gl> {
    pub entity: Option<Box<dyn SceneEntity + 'gl>>,
    pub name: String,
    pub new_name: String,
    pub id: usize,
}
