use glib_sys::{gpointer, GType};
use gobject_sys::{
    g_object_new, g_type_check_class_cast, g_type_check_instance_cast, g_type_class_peek_parent,
    g_type_module_register_type, g_type_module_use, g_type_register_static, GObject, GObjectClass,
    GTypeInfo, GTypeInstance, GTypeModule, G_TYPE_OBJECT,
};
use gtk_sys::{gtk_im_context_get_type, GtkIMContext, GtkIMContextClass, GtkIMContextInfo};
use libc::c_char;
use once_cell::sync::OnceCell;
use std::mem::size_of;
use std::ptr;

struct KimeIMContextClass {
    _parent: GtkIMContextClass,
}

#[repr(C)]
struct KimeIMContext {
    parent: GtkIMContext,
}

static KIME_TYPE_IM_CONTEXT: OnceCell<GType> = OnceCell::new();

unsafe fn register_module(module: *mut GTypeModule) {
    unsafe extern "C" fn im_context_class_init(class: gpointer, _data: gpointer) {
        let class = class.cast::<KimeIMContextClass>();

        let im_context_class = g_type_check_instance_cast(class.cast(), gtk_im_context_get_type())
            .cast::<GtkIMContextClass>();
        let gobject_class =
            g_type_check_class_cast(class.cast(), G_TYPE_OBJECT).cast::<GObjectClass>();

        (*im_context_class).set_client_window = None;
        (*im_context_class).filter_keypress = None;
        (*im_context_class).reset = None;
        (*im_context_class).get_preedit_string = None;
        (*im_context_class).focus_in = None;
        (*im_context_class).focus_out = None;
        (*im_context_class).set_cursor_location = None;
        (*im_context_class).set_use_preedit = None;
        (*im_context_class).set_surrounding = None;

        (*gobject_class).finalize = Some(im_context_instance_finalize);
    }

    unsafe extern "C" fn im_context_class_finalize(class: gpointer, _data: gpointer) {
        let _class = class.cast::<KimeIMContextClass>();
    }

    unsafe extern "C" fn im_context_instance_init(ctx: *mut GTypeInstance, _class: gpointer) {
        let _ctx = ctx.cast::<KimeIMContext>();
    }

    unsafe extern "C" fn im_context_instance_finalize(ctx: *mut GObject) {
        let ctx = ctx.cast::<KimeIMContext>();
        ctx.drop_in_place();
    }

    // GTypeInfo can't be static so just leak
    let info = Box::leak(Box::new(GTypeInfo {
        class_size: size_of::<KimeIMContextClass>() as _,
        base_init: None,
        base_finalize: None,
        class_init: Some(im_context_class_init),
        class_finalize: Some(im_context_class_finalize),
        class_data: ptr::null(),
        instance_size: size_of::<KimeIMContext>() as _,
        n_preallocs: 0,
        instance_init: Some(im_context_instance_init),
        value_table: ptr::null(),
    }));

    KIME_TYPE_IM_CONTEXT.get_or_init(|| {
        if module.is_null() {
            g_type_register_static(
                gtk_im_context_get_type(),
                b"KimeImContext\0".as_ptr().cast(),
                info,
                0,
            )
        } else {
            g_type_module_register_type(
                module,
                gtk_im_context_get_type(),
                b"KimeIMContext\0".as_ptr().cast(),
                info,
                0,
            )
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn im_module_init(module: *mut GTypeModule) {
    g_type_module_use(module);
    register_module(module);
}

#[no_mangle]
pub unsafe extern "C" fn im_module_exit() {}

#[no_mangle]
pub unsafe extern "C" fn im_module_list(contexts: *mut *mut *mut GtkIMContextInfo) {}

#[no_mangle]
pub unsafe extern "C" fn im_module_create(
    context_id: *const c_char,
) -> Option<ptr::NonNull<GtkIMContext>> {
    if !context_id.is_null() && libc::strcmp(context_id, b"kime".as_ptr() as _) == 0 {
        let obj = g_object_new(*KIME_TYPE_IM_CONTEXT.get()?, ptr::null());
        ptr::NonNull::new(obj.cast())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
