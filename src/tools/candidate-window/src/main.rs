use gtk::prelude::*;

fn activate(window: &gtk::ApplicationWindow) {
    if !window.display().backend().is_wayland() {
        eprintln!("Should be run on wayland backend.")
    }

    gtk_layer_shell::init_for_window(window);
    gtk_layer_shell::set_layer(window, gtk_layer_shell::Layer::Overlay);
    gtk_layer_shell::set_keyboard_mode(window, gtk_layer_shell::KeyboardMode::Exclusive);
    // gtk_layer_shell::set_margin(window, gtk_layer_shell::Edge::Right, 40);
    // gtk_layer_shell::set_margin(window, gtk_layer_shell::Edge::Bottom, 20);

    let anchors = [
        (gtk_layer_shell::Edge::Left, false),
        (gtk_layer_shell::Edge::Right, true),
        (gtk_layer_shell::Edge::Top, false),
        (gtk_layer_shell::Edge::Bottom, true),
    ];

    for (anchor, state) in anchors {
        gtk_layer_shell::set_anchor(window, anchor, state);
    }
}

use std::{
    io::{self, BufRead},
    rc::Rc,
};

struct CandidateApp {
    candidate_list: Rc<Vec<(String, String, gtk::Button)>>,
}

impl CandidateApp {
    pub fn run_candidate(&self, window: &gtk::ApplicationWindow) {
        let search_entry = gtk::SearchEntry::new();

        let scroll = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        scroll.set_size_request(500, 70);
        let list = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        list.set_layout(gtk::ButtonBoxStyle::Expand);

        scroll.add(&list);

        for (_, _, label) in self.candidate_list.iter() {
            list.add(label);
        }

        let se_weak = search_entry.downgrade();
        let list_weak = list.clone();
        window.connect_key_press_event(move |window, e| {
            let Some(search_entry) = se_weak.upgrade() else { return gtk::Inhibit(false); };
            search_entry.handle_event(e);

            match e.keyval() {
                gdk::keys::constants::Escape => {
                    window.close();
                }
                gdk::keys::constants::Return => {
                    if let Some(child) = list_weak.children().first() {
                        child.emit_by_name::<()>("clicked", &[]);
                    }
                }
                _ => {}
            }

            gtk::Inhibit(true)
        });

        let candidate_list_weak = Rc::downgrade(&self.candidate_list);
        let list_weak = list.downgrade();
        search_entry.connect_search_changed(move |search_entry| {
            let Some(list) = list_weak.upgrade() else { return; };
            let Some(candidate_list) = candidate_list_weak.upgrade() else { return; };

            for child in list.children() {
                list.remove(&child);
            }

            let text = search_entry.text();

            if text.is_empty() {
                for (_, _, label) in candidate_list.iter() {
                    list.add(label);
                }
            } else {
                for (_, value, label) in candidate_list.iter() {
                    if value.contains(text.as_str()) {
                        list.add(label);
                    }
                }
            }

            list.show_all();
        });

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        vbox.add(&search_entry);
        vbox.add(&scroll);

        window.add(&vbox);
        window.show_all();
    }
}

fn main() -> io::Result<()> {
    std::env::set_var("GDK_BACKEND", "wayland");
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

    let cadidate = CandidateApp {
        candidate_list: Rc::new(candidate_list),
    };

    let application = gtk::Application::new(None, Default::default());

    application.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_decorated(false);
        window.set_default_size(500, 600);
        window.set_title("kime");
        window.set_border_width(12);

        for (key, _, button) in cadidate.candidate_list.iter() {
            let window_weak = window.downgrade();
            let key_in = key.clone();
            button.connect_clicked(move |_| {
                let window = window_weak.upgrade().unwrap();
                print!("{key_in}");
                window.close();
            });
        }
        activate(&window);

        cadidate.run_candidate(&window);
    });

    application.run();

    Ok(())
}
