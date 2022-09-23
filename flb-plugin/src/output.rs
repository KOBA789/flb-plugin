//! Output Plugin bindings
//!
//! ```no_run
//! struct Hello;
//! impl flb_plugin::output::Plugin for Hello {
//!     const NAME: &'static CStr = const_cstr!("hello");
//!     const DESCRIPTION: &'static CStr = const_cstr!("hello plugin");
//!
//!     fn new(config: &output::Config) -> Self {
//!         let param = config.get_property(const_cstr!("param"));
//!         println!("[new] param: {:?}", param);
//!         Hello
//!     }
//!
//!     fn flush(&mut self, tag: &str, mut data: &[u8]) -> Result<(), flb_plugin::Error> {
//!         let value = rmpv::decode::read_value_ref(&mut data).unwrap();
//!         println!("[flush] tag: {tag}, data: {:?}", value);
//!         Ok(())
//!     }
//!
//!     fn exit(self) -> Result<(), flb_plugin::Error> {
//!         println!("[exit]");
//!         Ok(())
//!     }
//! }
//! flb_plugin::output_plugin_proxy!(Hello);
//! ```

use std::{
    ffi::{c_void, CStr},
    marker::PhantomData,
    os::raw::c_int,
    slice,
};

use flb_plugin_sys::{
    flb_plugin_proxy_def, flbgo_output_plugin, FLB_ERROR, FLB_OK, FLB_PROXY_GOLANG,
    FLB_PROXY_OUTPUT_PLUGIN, FLB_RETRY,
};

use crate::{instance_from_ctx, Error};

/// An accessor for plugin configurations
pub struct Config {
    plugin: *const flbgo_output_plugin,
}

impl Config {
    /// Returns the configuration property value.
    ///
    /// If the key is absent, this returns `None`.
    pub fn get_property(&self, key: &CStr) -> Option<&str> {
        unsafe {
            let plugin = self.plugin.as_ref()?;
            let output_get_property = (*(*plugin).api).output_get_property?;
            let value = output_get_property(key.as_ptr() as *mut i8, plugin.o_ins as *mut _);
            if value.is_null() {
                None
            } else {
                let cstr = CStr::from_ptr(value);
                cstr.to_str().ok()
            }
        }
    }
}

/// A trait for Fluent Bit output plugin
pub trait Plugin {
    /// The plugin name.
    const NAME: &'static CStr;

    /// The plugin description.
    const DESCRIPTION: &'static CStr;

    /// Creates a new plugin instance.
    fn new(config: &Config) -> Self;

    /// Handles data passed from Fluent Bit.
    ///
    /// `data` is a MessagePack byte buffer.
    fn flush(&mut self, tag: &str, data: &[u8]) -> Result<(), Error>;

    /// Cleans up the plugin instance.
    fn exit(self) -> Result<(), Error>;
}

/// A proxy object for [Plugin]
///
/// Use [crate::output_plugin_proxy] instead of using this directly.
pub struct Proxy<P> {
    plugin: PhantomData<P>,
}

impl<P> Proxy<P>
where
    P: Plugin,
{
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            plugin: PhantomData,
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn register(&self, def: *mut flb_plugin_proxy_def) -> c_int {
        let def = def.as_mut().unwrap();
        def.type_ = FLB_PROXY_OUTPUT_PLUGIN;
        def.proxy = FLB_PROXY_GOLANG;
        def.flags = 0;
        def.name = P::NAME.as_ptr() as *mut _;
        def.description = P::NAME.as_ptr() as *mut _;
        0
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn unregister(&self, _def: *mut flb_plugin_proxy_def) -> c_int {
        0
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn init(&self, plugin: *mut flbgo_output_plugin) -> c_int {
        let config = Config { plugin };
        let instance = P::new(&config);
        let ctx = Box::new(instance);
        let ctx = Box::into_raw(ctx);
        (*(*plugin).context).remote_context = ctx as *mut _;
        FLB_OK
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn flush(
        &self,
        ctx: *mut c_void,
        data: *const u8,
        len: c_int,
        tag: *const i8,
    ) -> c_int {
        let mut instance = match instance_from_ctx::<P>(ctx) {
            Some(instance) => instance,
            None => return FLB_ERROR,
        };
        let tag = CStr::from_ptr(tag);
        let tag = match tag.to_str() {
            Ok(s) => s,
            Err(_) => return FLB_ERROR,
        };
        let data = slice::from_raw_parts(data, len as usize);
        let ret = instance.flush(tag, data);
        Box::leak(instance);
        match ret {
            Ok(_) => FLB_OK,
            Err(Error::Error) => FLB_ERROR,
            Err(Error::Retry) => FLB_RETRY,
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn exit(&self, ctx: *mut c_void) -> c_int {
        let instance = match instance_from_ctx::<P>(ctx) {
            Some(instance) => instance,
            None => return FLB_ERROR,
        };
        let ret = instance.exit();
        match ret {
            Ok(_) => FLB_OK,
            Err(Error::Error) => FLB_ERROR,
            Err(Error::Retry) => FLB_RETRY,
        }
    }
}

/// Defines proxy functions (`FLBPluginRegister`, `FLBPluginInit`, ...)
#[macro_export]
macro_rules! output_plugin_proxy {
    ($proxy:path) => {
        const PROXY: $crate::output::Proxy<$proxy> = $crate::output::Proxy::new();

        #[allow(clippy::missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn FLBPluginRegister(
            def: *mut $crate::sys::flb_plugin_proxy_def,
        ) -> c_int {
            PROXY.register(def)
        }

        #[allow(clippy::missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn FLBPluginUnregister(
            def: *mut $crate::sys::flb_plugin_proxy_def,
        ) -> c_int {
            PROXY.unregister(def)
        }

        #[allow(clippy::missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn FLBPluginInit(
            plugin: *mut $crate::sys::flbgo_output_plugin,
        ) -> c_int {
            PROXY.init(plugin)
        }

        #[allow(clippy::missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn FLBPluginFlushCtx(
            ctx: *mut c_void,
            data: *const u8,
            len: c_int,
            tag: *const i8,
        ) -> c_int {
            PROXY.flush(ctx, data, len, tag)
        }

        #[allow(clippy::missing_safety_doc)]
        #[no_mangle]
        pub unsafe extern "C" fn FLBPluginExitCtx(ctx: *mut c_void) -> c_int {
            PROXY.exit(ctx)
        }
    };
}
