use gdk_sys::{
    gdk_event_copy, gdk_event_put, gdk_keyval_to_unicode, gdk_window_get_user_data, GdkColor,
    GdkEvent, GdkEventKey, GdkWindow, GDK_CONTROL_MASK, GDK_KEY_PRESS, GDK_MOD1_MASK,
    GDK_MOD2_MASK, GDK_MOD3_MASK, GDK_MOD4_MASK, GDK_MOD5_MASK, GDK_SHIFT_MASK,
};
use glib_sys::{g_malloc0, g_strcmp0, g_strdup, gboolean, gpointer, GType, GFALSE, GTRUE};
use gobject_sys::{
    g_object_new, g_object_ref, g_object_unref, g_signal_emit, g_signal_lookup,
    g_type_check_class_cast, g_type_check_instance_cast, g_type_check_instance_is_a,
    g_type_module_register_type, g_type_module_use, g_type_register_static, GObject, GObjectClass,
    GTypeClass, GTypeInfo, GTypeInstance, GTypeModule, G_TYPE_OBJECT,
};
use gtk_sys::{
    gtk_im_context_get_type, gtk_style_context_lookup_color, gtk_widget_get_style_context,
    gtk_widget_get_type, gtk_window_get_type, GtkIMContext, GtkIMContextClass, GtkIMContextInfo,
};
use once_cell::sync::{Lazy, OnceCell};
use pango_sys::PangoAttrList;
use std::os::raw::{c_char, c_int, c_uint};
use std::ptr::{self, NonNull};
use std::{
    mem::{size_of, MaybeUninit},
    os::raw::c_double,
};

use kime_engine::{Config, InputEngine, InputResult};

const FORWARDED_MASK: c_uint = 1 << 25;
const SKIP_MASK: c_uint =
    GDK_MOD1_MASK | GDK_MOD2_MASK | GDK_MOD3_MASK | GDK_MOD4_MASK | GDK_MOD5_MASK | FORWARDED_MASK;

#[repr(transparent)]
struct TypeInfoWrapper(GTypeInfo);

unsafe impl Send for TypeInfoWrapper {}
unsafe impl Sync for TypeInfoWrapper {}

#[repr(transparent)]
struct ContextInfoWrapper(GtkIMContextInfo);

unsafe impl Send for ContextInfoWrapper {}
unsafe impl Sync for ContextInfoWrapper {}

macro_rules! cs {
    ($text:expr) => {
        concat!($text, "\0").as_ptr().cast::<c_char>()
    };
}

const DEFAULT_HL_FG: GdkColor = GdkColor {
    pixel: 0,
    red: 0xffff,
    green: 0xffff,
    blue: 0xffff,
};

const DEFAULT_HL_BG: GdkColor = GdkColor {
    pixel: 0,
    red: 0x43ff,
    green: 0xacff,
    blue: 0xe8ff,
};

fn put_event(key: &mut GdkEventKey) {
    #[cfg(debug_assertions)]
    eprintln!("put_event");

    key.state |= FORWARDED_MASK;
    unsafe {
        let e = gdk_event_copy(key as *const GdkEventKey as *const GdkEvent);
        gdk_event_put(e);
    }
}

unsafe fn lookup_color(
    context: *mut gtk_sys::GtkStyleContext,
    name: *const c_char,
) -> Option<GdkColor> {
    let mut rgba = MaybeUninit::uninit();
    if gtk_style_context_lookup_color(context, name, rgba.as_mut_ptr()) == GTRUE {
        let rgba = rgba.assume_init();

        fn convert_color(c: c_double) -> u16 {
            (c.max(0.0).min(1.0) * u16::MAX as c_double) as u16
        }

        Some(GdkColor {
            pixel: 0,
            red: convert_color(rgba.red),
            blue: convert_color(rgba.blue),
            green: convert_color(rgba.green),
        })
    } else {
        None
    }
}

struct KimeIMSignals {
    commit: c_uint,
    preedit_start: c_uint,
    preedit_changed: c_uint,
    preedit_end: c_uint,
}

impl KimeIMSignals {
    pub unsafe fn new(class: *mut KimeIMContextClass) -> Self {
        let ty = type_of_class(class.cast());

        macro_rules! sig {
            ($($name:ident),+) => {
                Self { $($name: g_signal_lookup(cs!(stringify!($name)), ty),)+ }
            }
        }

        sig!(commit, preedit_start, preedit_changed, preedit_end)
    }
}

static SIGNALS: OnceCell<KimeIMSignals> = OnceCell::new();
static CONFIG: Lazy<Config> = Lazy::new(|| Config::load_from_config_dir().unwrap_or_default());

#[repr(C)]
struct KimeIMContextClass {
    _parent: GtkIMContextClass,
}

#[repr(C)]
struct KimeIMContext {
    parent: GtkIMContext,
    client_window: Option<NonNull<GdkWindow>>,
    engine: InputEngine,
    preedit_visible: bool,
}

impl KimeIMContext {
    pub fn as_obj(&mut self) -> *mut GObject {
        &mut self.parent.parent_instance
    }

    pub fn filter_keypress(&mut self, key: &mut GdkEventKey) -> bool {
        let code = match kime_engine::KeyCode::from_hardward_code(key.hardware_keycode) {
            Some(code) => code,
            None => {
                return self.bypass(key);
            }
        };

        let ret = self.engine.press_key(
            kime_engine::Key::new(
                code,
                key.state & GDK_SHIFT_MASK != 0,
                key.state & GDK_CONTROL_MASK != 0,
            ),
            &CONFIG,
        );

        dbg!(ret);

        match ret {
            InputResult::Commit(c) => {
                self.update_preedit(false);
                self.commit(c);
                true
            }
            InputResult::CommitCommit(f, s) => {
                self.update_preedit(false);
                self.commit(f);
                self.commit(s);
                true
            }
            InputResult::CommitBypass(c) => {
                self.update_preedit(false);
                self.commit(c);
                put_event(key);
                true
            }
            InputResult::CommitPreedit(c, _p) => {
                self.commit(c);
                self.update_preedit(true);
                true
            }
            InputResult::Preedit(_p) => {
                self.update_preedit(true);
                true
            }
            InputResult::ClearPreedit => {
                self.update_preedit(false);
                true
            }
            InputResult::Bypass => false,
            InputResult::Consume => true,
        }
    }

    pub fn reset(&mut self) {
        match self.engine.reset() {
            Some(c) => {
                self.update_preedit(false);
                self.commit(c);
            }
            _ => {}
        }
    }

    pub fn commit_event(&mut self, key: &GdkEventKey) -> gboolean {
        if CONFIG.gtk_commit_english && key.state & GDK_CONTROL_MASK == 0 {
            let c = unsafe { std::char::from_u32_unchecked(gdk_keyval_to_unicode(key.keyval)) };

            if !c.is_control() {
                self.commit(c);
                return GTRUE;
            }
        }

        GFALSE
    }

    pub fn update_preedit(&mut self, visible: bool) {
        #[cfg(debug_assertions)]
        eprintln!("update_preedit: {}", visible);
        if self.preedit_visible != visible {
            self.preedit_visible = visible;

            if visible {
                unsafe {
                    g_signal_emit(self.as_obj(), SIGNALS.get().unwrap().preedit_start, 0);
                    g_signal_emit(self.as_obj(), SIGNALS.get().unwrap().preedit_changed, 0);
                }
            } else {
                unsafe {
                    g_signal_emit(self.as_obj(), SIGNALS.get().unwrap().preedit_changed, 0);
                    g_signal_emit(self.as_obj(), SIGNALS.get().unwrap().preedit_end, 0);
                }
            }
        } else {
            // visible update
            if visible {
                unsafe {
                    g_signal_emit(self.as_obj(), SIGNALS.get().unwrap().preedit_changed, 0);
                }
            // invisible noop
            } else {
            }
        }
    }

    pub fn bypass(&mut self, key: &mut GdkEventKey) -> bool {
        match self.engine.reset() {
            Some(c) => {
                self.update_preedit(false);
                self.commit(c);
                put_event(key);
                true
            }
            _ => false,
        }
    }

    pub fn commit(&mut self, c: char) {
        #[cfg(debug_assertions)]
        eprintln!("commit: {}", c);
        let mut buf = [0; 8];
        c.encode_utf8(&mut buf);
        unsafe {
            g_signal_emit(
                self.as_obj(),
                SIGNALS.get().unwrap().commit,
                0,
                buf.as_ptr(),
            );
        }
    }
}

static KIME_TYPE_IM_CONTEXT: OnceCell<GType> = OnceCell::new();

unsafe fn type_of_class(class: *mut GTypeClass) -> GType {
    (*class).g_type
}

unsafe fn register_module(module: *mut GTypeModule) {
    unsafe extern "C" fn im_context_class_init(class: gpointer, _data: gpointer) {
        let class = class.cast::<KimeIMContextClass>();

        let im_context_class = g_type_check_class_cast(class.cast(), gtk_im_context_get_type())
            .cast::<GtkIMContextClass>()
            .as_mut()
            .unwrap();
        let gobject_class =
            g_type_check_class_cast(class.cast(), G_TYPE_OBJECT).cast::<GObjectClass>();

        im_context_class.set_client_window = Some(set_client_window);
        im_context_class.filter_keypress = Some(filter_keypress);
        im_context_class.reset = Some(reset_im);
        im_context_class.get_preedit_string = Some(get_preedit_string);
        im_context_class.focus_in = Some(focus_in);
        im_context_class.focus_out = Some(focus_out);
        im_context_class.set_cursor_location = None;
        im_context_class.set_use_preedit = None;
        im_context_class.set_surrounding = None;

        SIGNALS.get_or_init(|| KimeIMSignals::new(class));

        (*gobject_class).finalize = Some(im_context_instance_finalize);
    }

    unsafe extern "C" fn focus_in(_ctx: *mut GtkIMContext) {}

    unsafe extern "C" fn focus_out(ctx: *mut GtkIMContext) {
        reset_im(ctx);
    }

    unsafe extern "C" fn reset_im(ctx: *mut GtkIMContext) {
        let ctx = ctx.cast::<KimeIMContext>().as_mut().unwrap();
        ctx.reset();
    }

    unsafe extern "C" fn filter_keypress(
        ctx: *mut GtkIMContext,
        key_ptr: *mut GdkEventKey,
    ) -> gboolean {
        let ctx = ctx.cast::<KimeIMContext>().as_mut().unwrap();
        let key = key_ptr.as_mut().unwrap();

        if key.type_ != GDK_KEY_PRESS {
            GFALSE
        } else if key.state & FORWARDED_MASK != 0 {
            #[cfg(debug_assertions)]
            eprintln!("FORWARDED: {}", key.keyval);
            ctx.commit_event(key)
        } else if key.state & SKIP_MASK != 0 {
            ctx.bypass(key).into()
        } else if ctx.filter_keypress(key) {
            GTRUE
        } else {
            ctx.commit_event(key)
        }
    }

    unsafe extern "C" fn get_preedit_string(
        ctx: *mut GtkIMContext,
        out: *mut *mut c_char,
        attrs: *mut *mut PangoAttrList,
        cursor_pos: *mut c_int,
    ) {
        let ctx = ctx.cast::<KimeIMContext>().as_mut().unwrap();
        let ch = ctx.engine.preedit_char();
        let mut str_len = 0;

        if !out.is_null() {
            // Noting to display
            if ch == '\0' {
                if !cursor_pos.is_null() {
                    cursor_pos.write(0);
                }
                out.write(g_strdup(cs!("")));
            } else {
                if !cursor_pos.is_null() {
                    cursor_pos.write(1);
                }
                str_len = ch.len_utf8();
                let s = g_malloc0(str_len + 1).cast::<c_char>();
                ch.encode_utf8(std::slice::from_raw_parts_mut(s.cast(), str_len));
                s.add(str_len).write(0);
                out.write(s);
            }
        }

        if !attrs.is_null() {
            attrs.write(pango_sys::pango_attr_list_new());

            if !out.is_null() && ch != '\0' {
                let attr = pango_sys::pango_attr_underline_new(pango_sys::PANGO_UNDERLINE_SINGLE);
                (*attr).start_index = 0;
                (*attr).end_index = str_len as _;
                pango_sys::pango_attr_list_insert(attrs.read(), attr);

                if let Some(window) = ctx.client_window {
                    let mut widget = MaybeUninit::uninit();
                    gdk_window_get_user_data(window.as_ptr(), widget.as_mut_ptr());
                    let widget = widget.assume_init();

                    if g_type_check_instance_is_a(widget.cast(), gtk_widget_get_type()) == GTRUE
                        && g_type_check_instance_is_a(widget.cast(), gtk_window_get_type())
                            == GFALSE
                    {
                        let widget = widget.cast();
                        let style_ctx = gtk_widget_get_style_context(widget);

                        let fg = lookup_color(style_ctx, cs!("theme_selected_fg_color"))
                            .unwrap_or(DEFAULT_HL_FG);
                        let bg = lookup_color(style_ctx, cs!("theme_selected_bg_color"))
                            .unwrap_or(DEFAULT_HL_BG);

                        let fg_attr =
                            pango_sys::pango_attr_foreground_new(fg.red, fg.green, fg.blue);
                        (*fg_attr).start_index = 0;
                        (*fg_attr).end_index = str_len as _;

                        let bg_attr =
                            pango_sys::pango_attr_background_new(bg.red, bg.green, bg.blue);
                        (*bg_attr).start_index = 0;
                        (*bg_attr).end_index = str_len as _;

                        pango_sys::pango_attr_list_insert(attrs.read(), fg_attr);
                        pango_sys::pango_attr_list_insert(attrs.read(), bg_attr);
                    }
                }
            }
        }
    }

    unsafe extern "C" fn set_client_window(ctx: *mut GtkIMContext, window: *mut GdkWindow) {
        let ctx = ctx.cast::<KimeIMContext>().as_mut().unwrap();
        let window = NonNull::new(window);

        if let Some(prev_win) = ctx.client_window {
            g_object_unref(prev_win.as_ptr().cast());
        }

        if let Some(win) = window {
            g_object_ref(win.as_ptr().cast());
        }

        ctx.client_window = window;
    }

    unsafe extern "C" fn im_context_class_finalize(class: gpointer, _data: gpointer) {
        let _class = class.cast::<KimeIMContextClass>();
    }

    unsafe extern "C" fn im_context_instance_init(ctx: *mut GTypeInstance, _class: gpointer) {
        let parent = ctx.cast::<GtkIMContext>();

        ctx.cast::<KimeIMContext>().write(KimeIMContext {
            parent: parent.read(),
            client_window: None,
            engine: InputEngine::new(),
            preedit_visible: false,
        });
    }

    unsafe extern "C" fn im_context_instance_finalize(ctx: *mut GObject) {
        let ctx = ctx.cast::<KimeIMContext>();
        ctx.drop_in_place();
    }

    static INFO: TypeInfoWrapper = TypeInfoWrapper(GTypeInfo {
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
    });

    KIME_TYPE_IM_CONTEXT.get_or_init(|| {
        if module.is_null() {
            g_type_register_static(gtk_im_context_get_type(), cs!("KimeImContext"), &INFO.0, 0)
        } else {
            g_type_module_register_type(
                module,
                gtk_im_context_get_type(),
                cs!("KimeIMContext"),
                &INFO.0,
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
pub unsafe extern "C" fn im_module_list(
    contexts: *mut *const *const GtkIMContextInfo,
    n_contexts: *mut c_int,
) {
    static INFO: ContextInfoWrapper = ContextInfoWrapper(GtkIMContextInfo {
        context_id: cs!("kime"),
        context_name: cs!("Kime (Korean IME)"),
        domain: cs!("kime"),
        domain_dirname: cs!("/usr/share/locale"),
        default_locales: cs!("ko:*"),
    });

    static INFOS: &[&ContextInfoWrapper] = &[&INFO];

    // SAFETY: *const &ContextInfoWrapper -> *const *const GtkIMContextInfo
    // & == *const, ContextInfoWrapper == GtkIMContextInfo(transparent)
    contexts.write(INFOS.as_ptr().cast());
    n_contexts.write(INFOS.len() as _);
}

#[no_mangle]
pub unsafe extern "C" fn im_module_create(
    context_id: *const c_char,
) -> Option<ptr::NonNull<GtkIMContext>> {
    if !context_id.is_null() && g_strcmp0(context_id, cs!("kime")) == 0 {
        let ty = *KIME_TYPE_IM_CONTEXT.get()?;
        let obj = g_object_new(ty, ptr::null());
        ptr::NonNull::new(g_type_check_instance_cast(obj.cast(), ty).cast())
    } else {
        None
    }
}
