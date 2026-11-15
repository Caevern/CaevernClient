use wasmtime::Caller;

use crate::modules::host::HostState;

pub fn game_log(mut caller: Caller<'_, HostState>, ptr: u32, len: u32) {
    let memory = match caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
    {
        Some(memory) => memory,
        None => {
            eprintln!("[MOD] No memory export");
            return;
        }
    };

    let data = memory.data(&caller);

    let start = ptr as usize;
    let end = start + len as usize;

    let bytes = &data[start..end];

    match std::str::from_utf8(bytes) {
        Ok(message) => println!("[MOD] {message}"),
        Err(_) => eprintln!("[MOD] Invalid UTF-8"),
    }
}
