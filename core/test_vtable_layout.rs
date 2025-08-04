use lao_plugin_api::*;
use std::mem;

fn main() {
    println!("PluginVTable size: {} bytes", mem::size_of::<PluginVTable>());
    println!("PluginVTable alignment: {} bytes", mem::align_of::<PluginVTable>());
    
    println!("\nField offsets:");
    println!("version: offset {}", unsafe { 
        &(*(std::ptr::null::<PluginVTable>())).version as *const u32 as usize 
    });
    println!("name: offset {}", unsafe { 
        &(*(std::ptr::null::<PluginVTable>())).name as *const _ as usize 
    });
    println!("run: offset {}", unsafe { 
        &(*(std::ptr::null::<PluginVTable>())).run as *const _ as usize 
    });
    println!("free_output: offset {}", unsafe { 
        &(*(std::ptr::null::<PluginVTable>())).free_output as *const _ as usize 
    });
    println!("run_with_buffer: offset {}", unsafe { 
        &(*(std::ptr::null::<PluginVTable>())).run_with_buffer as *const _ as usize 
    });
    println!("get_metadata: offset {}", unsafe { 
        &(*(std::ptr::null::<PluginVTable>())).get_metadata as *const _ as usize 
    });
    println!("validate_input: offset {}", unsafe { 
        &(*(std::ptr::null::<PluginVTable>())).validate_input as *const _ as usize 
    });
    println!("get_capabilities: offset {}", unsafe { 
        &(*(std::ptr::null::<PluginVTable>())).get_capabilities as *const _ as usize 
    });
    
    // Create a dummy vtable to see what the first field contains
    let dummy_vtable = PluginVTable {
        version: 1,
        name: || std::ptr::null(),
        run: |_| PluginOutput { text: std::ptr::null_mut() },
        free_output: |_| {},
        run_with_buffer: |_, _, _| 0,
        get_metadata: || PluginMetadata {
            name: std::ptr::null(),
            version: std::ptr::null(),
            description: std::ptr::null(),
            author: std::ptr::null(),
            dependencies: std::ptr::null(),
            tags: std::ptr::null(),
            input_schema: std::ptr::null(),
            output_schema: std::ptr::null(),
            capabilities: std::ptr::null(),
        },
        validate_input: |_| true,
        get_capabilities: || std::ptr::null(),
    };
    
    println!("\nDummy vtable version: {}", dummy_vtable.version);
    println!("Dummy vtable get_metadata pointer: {:?}", dummy_vtable.get_metadata);
} 