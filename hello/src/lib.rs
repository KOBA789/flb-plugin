use std::{
    ffi::{c_void, CStr},
    os::raw::c_int,
};

use flb_plugin::output;

macro_rules! const_cstr {
    ($s:literal) => {
        unsafe { CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes()) }
    };
}

struct Hello;
impl output::Plugin for Hello {
    const NAME: &'static CStr = const_cstr!("hello");
    const DESCRIPTION: &'static CStr = const_cstr!("hello plugin");

    fn new(config: &output::Config) -> Self {
        let param = config.get_property(const_cstr!("param"));
        println!("[new] param: {:?}", param);
        Hello
    }

    fn flush(&mut self, tag: &str, mut data: &[u8]) -> Result<(), flb_plugin::Error> {
        let value = rmpv::decode::read_value_ref(&mut data).unwrap();
        println!("[flush] tag: {tag}, data: {:?}", value);
        Ok(())
    }

    fn exit(self) -> Result<(), flb_plugin::Error> {
        println!("[exit]");
        Ok(())
    }
}

flb_plugin::output_plugin_proxy!(Hello);
