use crate::resource_local_registry::{Resource, RESOURCE_LOCAL_REGISTRY};
use crate::resources::{
    RESOURCE_DESKTOP_ID, RESOURCE_FILESYSTEM_ITEM_CURRENT_DIR_TAG, RESOURCE_FILESYSTEM_ITEM_ID,
    RESOURCE_FILESYSTEM_ITEM_ROOT_TAG,
};
use crate::typed_value::{KeyedTypedValue, TypedValue};
use crate::uuid::Uuid;
use std::collections::HashMap;
use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;

pub fn prepare_filesystem() {
    let process_id = std::process::id() as u128;
    let sythetic_process_id = 0x30312746_893c_4654_a9b5_000000000000;

    let root_directory_id = Some(Uuid::from_u128(0x200000001)); //tmp, do random gen
    let current_directory_id = Some(Uuid::from_u128(0x200000002)); //tmp, do random gen
    let current_directory_resource = Arc::new(Resource {
        uuid: current_directory_id.clone().unwrap(),
        resource_type: RESOURCE_FILESYSTEM_ITEM_ID,
        name: "Current directory".to_string(),
        methods: generateMethods(String::from(".")),
        tags: vec![RESOURCE_FILESYSTEM_ITEM_CURRENT_DIR_TAG],
        ..Default::default()
    });
    // let root_directory_resource = Arc::new(Resource {
    //     uuid: root_directory_id.clone().unwrap(),
    //     resource_type: RESOURCE_FILESYSTEM_ITEM_ID,
    //     name: "Desktop".to_string(),
    //     methods: generateMethods(String::from("/")),
    //     tags: vec![RESOURCE_FILESYSTEM_ITEM_ROOT_TAG],
    // });
    RESOURCE_LOCAL_REGISTRY.lock().unwrap().insert(
        current_directory_id.clone().unwrap(),
        current_directory_resource.clone(),
    );
    Arc::get_mut(
        RESOURCE_LOCAL_REGISTRY
            .lock()
            .unwrap()
            .get_mut(&Uuid::from_u128(sythetic_process_id + process_id))
            .unwrap(),
    )
    .unwrap()
    .connected_resources
    .push(current_directory_resource.clone());

    // registry.insert(root_directory_id.clone().unwrap(), root_directory_resource);
}
pub fn generateMethods(localPath: String) -> HashMap<String, fn(TypedValue) -> TypedValue> {
    let mut methods: HashMap<String, fn(TypedValue) -> TypedValue> = HashMap::new();
    methods.insert("getChildren".to_string(), |text| {
        let mut ret = Vec::new();
        for entry in std::fs::read_dir(".").unwrap() {
            let entry = entry.unwrap();
            ret.push(TypedValue::structure(vec![
                KeyedTypedValue::from("name".to_string(), TypedValue::string(entry.file_name().to_string_lossy().to_string())),
                KeyedTypedValue::from("isDirectory".to_string(), TypedValue::bool(entry.file_type().unwrap().is_dir())),
                KeyedTypedValue::from("isFile".to_string(), TypedValue::bool(entry.file_type().unwrap().is_file())),
            ].as_slice()));
        }
        return TypedValue::vector(ret.as_slice());
    });
    return methods;
}
