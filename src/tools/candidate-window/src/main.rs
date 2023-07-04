use gtk::prelude::*;

fn activate(window: &gtk::ApplicationWindow) {
    if !window.display().backend().is_wayland() {
        eprintln!("Should be run on wayland backend.")
    }

    gtk_layer_shell::init_for_window(window);
    gtk_layer_shell::set_layer(window, gtk_layer_shell::Layer::Overlay);
    gtk_layer_shell::set_keyboard_mode(window, gtk_layer_shell::KeyboardMode::Exclusive);
    gtk_layer_shell::set_margin(window, gtk_layer_shell::Edge::Left, 400);
    gtk_layer_shell::set_margin(window, gtk_layer_shell::Edge::Top, 200);

    let anchors = [
        (gtk_layer_shell::Edge::Left, true),
        (gtk_layer_shell::Edge::Right, false),
        (gtk_layer_shell::Edge::Top, true),
        (gtk_layer_shell::Edge::Bottom, false),
    ];

    for (anchor, state) in anchors {
        gtk_layer_shell::set_anchor(window, anchor, state);
    }
}

use std::{
    cell::Cell,
    io::{self, BufRead},
    rc::Rc,
};

const PAGE_SIZE: usize = 9;

struct CandidateApp {
    candidate_list: Rc<Vec<(String, String, gtk::Button)>>,
}

impl CandidateApp {
    pub fn run_candidate(&self, window: &gtk::ApplicationWindow) {
        let candidate_list = Rc::clone(&self.candidate_list);
        let list = gtk::ButtonBox::new(gtk::Orientation::Vertical);
        list.set_margin(0);
        list.set_layout(gtk::ButtonBoxStyle::Expand);

        for (_, _, label) in &self.candidate_list[..PAGE_SIZE] {
            list.add(label);
        }
        let max_page_index = if candidate_list.len() % PAGE_SIZE == 0 {
            (candidate_list.len() / PAGE_SIZE) - 1
        } else {
            candidate_list.len() / PAGE_SIZE
        };
        let page_index = Cell::new(0usize);

        let list_weak = list.clone();
        window.connect_key_press_event(move |window, e| {
            match e.keyval() {
                gdk::keys::constants::Escape => {
                    window.close();
                }
                gdk::keys::constants::Return => {
                    if let Some(child) = list_weak.children().first() {
                        child.emit_by_name::<()>("clicked", &[]);
                    }
                }
                gdk::keys::constants::uparrow | gdk::keys::constants::Up => {
                    if let Some(new_page_index) = page_index.get().checked_sub(1) {
                        for child in list_weak.children() {
                            list_weak.remove(&child);
                        }
                        page_index.set(new_page_index);
                        for (_, _, btn) in &candidate_list
                            [page_index.get() * PAGE_SIZE..(page_index.get() + 1) * PAGE_SIZE]
                        {
                            list_weak.add(btn);
                        }
                        list_weak.show_all();
                    }
                }
                gdk::keys::constants::downarrow | gdk::keys::constants::Down => {
                    if page_index.get() != max_page_index {
                        let new_page_index = page_index.get() + 1;
                        for child in list_weak.children() {
                            list_weak.remove(&child);
                        }
                        page_index.set(new_page_index);
                        for (_, _, btn) in &candidate_list[page_index.get() * PAGE_SIZE
                            ..((page_index.get() + 1) * PAGE_SIZE).min(candidate_list.len())]
                        {
                            list_weak.add(btn);
                        }
                        list_weak.show_all();
                    }
                }
                gdk::keys::constants::_1
                | gdk::keys::constants::_2
                | gdk::keys::constants::_3
                | gdk::keys::constants::_4
                | gdk::keys::constants::_5
                | gdk::keys::constants::_6
                | gdk::keys::constants::_7
                | gdk::keys::constants::_8
                | gdk::keys::constants::_9 => {
                    let index = *e.keyval() - *gdk::keys::constants::_1;
                    if let Some(child) = list_weak.children().get(index as usize) {
                        child.emit_by_name::<()>("clicked", &[]);
                    }
                }
                _ => {}
            }

            gtk::Inhibit(true)
        });

        window.add(&list);
        window.show_all();
    }
}

fn main() -> io::Result<()> {
    if std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland") {
        std::env::set_var("GDK_BACKEND", "wayland");
    }
    gtk::init().ok();

    let mut buf = String::with_capacity(4096);
    let stdin = io::stdin();

    let mut candidate_list = Vec::new();
    let mut stdin_lock = stdin.lock();

    macro_rules! read {
        ($ret:ident) => {
            let len = stdin_lock.read_line(&mut buf)?;
            if len == 0 {
                break;
            }
            let $ret = buf.trim_end_matches('\n').to_string();
            buf.clear();
        };
    }

    loop {
        read!(key);
        read!(value);
        let button = gtk::Button::new();
        let label = gtk::Label::new(Some(&format!("{key}\n{value}")));
        label.set_justify(gtk::Justification::Center);
        button.add(&label);
        candidate_list.push((key, value, button));
    }

    let candidate = CandidateApp {
        candidate_list: Rc::new(candidate_list),
    };

    let application = gtk::Application::new(None, Default::default());

    application.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_decorated(false);
        window.set_default_size(500, 600);

        for (key, _, button) in candidate.candidate_list.iter() {
            let window_weak = window.downgrade();
            let key_in = key.clone();
            button.connect_clicked(move |_| {
                let window = window_weak.upgrade().unwrap();
                print!("{key_in}");
                window.close();
            });
        }
        activate(&window);

        candidate.run_candidate(&window);
    });

    application.run();

    Ok(())
}
