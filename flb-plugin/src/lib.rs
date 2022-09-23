use std::os::raw::c_void;

pub mod output;
pub use flb_plugin_sys as sys;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Error {
    Error,
    Retry,
}

pub(crate) unsafe fn instance_from_ctx<P>(ctx: *mut c_void) -> Option<Box<P>> {
    if ctx.is_null() {
        return None;
    }
    let ctx = ctx as *mut P;
    let instance = Box::from_raw(ctx);
    Some(instance)
}
