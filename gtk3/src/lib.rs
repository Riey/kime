use gobject_sys::GTypeModule;
use gtk_sys::{GtkIMContext, GtkIMContextInfo};
use std::os::raw::c_char;

#[no_mangle]
pub fn im_module_init(module: Option<&mut GTypeModule>) {
    println!("{:?}", module);
}

#[no_mangle]
pub fn im_module_exit() {}

#[no_mangle]
pub fn im_module_list(contexts: *mut *mut *mut GtkIMContextInfo) {}

#[no_mangle]
pub fn im_module_create(context_id: *mut c_char) -> *mut GtkIMContext {
    std::ptr::null_mut()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
