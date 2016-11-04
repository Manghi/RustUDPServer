use std::fmt;

#[derive(Debug)]
pub enum DebugPrint {
    COMMON      = 0x1000,
    NETWORK     = 0x1001,
    PACKET      = 0x1002,

    GRAPHICS    = 0x2000,

    LIBCONWAY   = 0x3000,

    AUDIO       = 0x4000,

    MAXDEBUGPRINTTYPES = 0xFFFF
}

impl fmt::Display for DebugPrint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[allow(dead_code)]
const DEBUG_PRINT_ENABLED : bool = true;

pub fn debug_println(debug_type: DebugPrint, module: &str, message: &str) {

    if DEBUG_PRINT_ENABLED {
        let debug_string = format!("[{:?}] {}: {}", debug_type, module, message);

        let debug_str = debug_string.as_str();

        println!("{}", debug_str);
    }

}
