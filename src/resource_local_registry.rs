use std::cell::RefCell;
use crate::uuid::Uuid;
use std::collections::HashMap;
use std::iter::Map;
use std::rc::Rc;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::typed_value::TypedValue;

pub static RESOURCE_LOCAL_REGISTRY: Lazy<Mutex<HashMap<Uuid, Resource>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});


pub struct Resource {
    pub uuid: Uuid,
    pub resource_type: Uuid,
    pub name: String,
    pub methods: HashMap<String, fn()->TypedValue>
}
