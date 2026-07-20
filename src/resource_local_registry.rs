use std::cell::RefCell;
use crate::uuid::Uuid;
use std::collections::HashMap;
use std::iter::Map;
use std::rc::Rc;
use std::sync::{Mutex, Arc};
use once_cell::sync::Lazy;
use crate::typed_value::TypedValue;

pub static RESOURCE_LOCAL_REGISTRY: Lazy<Mutex<HashMap<Uuid, Arc<Resource>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});


pub struct Resource {
    pub uuid: Uuid,
    pub resource_type: Uuid,
    pub name: String,
    pub methods: HashMap<String, fn(TypedValue)->TypedValue>,
    pub tags: std::vec::Vec<Uuid>,
    pub connected_resources:Vec<Arc<Resource>>
}
impl Default for Resource{
    fn default() -> Self {
        Resource {
            uuid: Uuid::from_u128(0),
            resource_type: Uuid::from_u128(0),
            name: String::new(),
            methods: HashMap::new(),
            tags: Vec::new(),
            connected_resources: Vec::new(),
        }
    }
}