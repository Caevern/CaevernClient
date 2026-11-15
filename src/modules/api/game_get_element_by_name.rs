use wasmtime::Caller;

use crate::modules::host::HostState;

pub fn game_get_element_by_name(mut caller: Caller<'_, HostState>, ptr: u32, len: u32) -> u32 {
    let memory = match caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
    {
        Some(memory) => memory,
        None => {
            eprintln!("[MOD] No memory export");
            return 0;
        }
    };

    let data = memory.data(&caller);

    let start = ptr as usize;
    let end = start + len as usize;

    let bytes = &data[start..end];

    match std::str::from_utf8(bytes) {
        Ok(name) => {
            let data = caller.data();
            let objects = data.world.borrow_mut();

            objects
                .get_objects()
                .iter()
                .find(|element| element.1.get_name() == name)
                .map(|element| *element.0 as u32)
                .unwrap_or(0)
        }
        Err(_) => {
            eprintln!("[MOD] Invalid UTF-8");
            0
        }
    }
}
