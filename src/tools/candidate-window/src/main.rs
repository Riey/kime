use std::{
    collections::BTreeMap,
    io::{self, BufRead, Stdout, Write},
    sync::Arc,
};

use egui::Widget;

const PAGE_SIZE: usize = 10;

#[derive(Default)]
struct KeyState {
    left: bool,
    right: bool,
}

struct CandidateApp {
    stdout: Stdout,
    key_state: KeyState,
    page_index: usize,
    max_page_index: usize,
    candidate_list: Vec<(String, String)>,
}

impl eframe::App for CandidateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_down(egui::Key::Escape)) || ctx.input(|i| i.key_down(egui::Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        macro_rules! num_hotkey {
            ($k:expr, $n:expr) => {
                if ctx.input(|i| i.key_down($k)) {
                    self.page_index = $n;
                }
            };
        }

        num_hotkey!(egui::Key::Num1, 0);
        num_hotkey!(egui::Key::Num2, 1);
        num_hotkey!(egui::Key::Num3, 2);
        num_hotkey!(egui::Key::Num4, 3);
        num_hotkey!(egui::Key::Num5, 4);
        num_hotkey!(egui::Key::Num6, 5);
        num_hotkey!(egui::Key::Num7, 6);
        num_hotkey!(egui::Key::Num8, 7);
        num_hotkey!(egui::Key::Num9, 8);
        num_hotkey!(egui::Key::Num0, 9);

        if ctx.input(|i| i.key_down(egui::Key::ArrowLeft))
            || ctx.input(|i| i.key_down(egui::Key::H))
        {
            if !self.key_state.left {
                self.page_index = self.page_index.saturating_sub(1);
                self.key_state.left = true;
            }
        }

        if ctx.input(|i| i.key_released(egui::Key::ArrowLeft))
            || ctx.input(|i| i.key_released(egui::Key::H))
        {
            self.key_state.left = false;
        }

        if ctx.input(|i| i.key_down(egui::Key::ArrowRight))
            || ctx.input(|i| i.key_down(egui::Key::L))
        {
            if !self.key_state.right {
                self.page_index = self.page_index.saturating_add(1).min(self.max_page_index);
                self.key_state.right = true;
            }
        }

        if ctx.input(|i| i.key_released(egui::Key::ArrowRight))
            || ctx.input(|i| i.key_released(egui::Key::L))
        {
            self.key_state.right = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let from = self.page_index * PAGE_SIZE;
                let to = (from + PAGE_SIZE).min(self.candidate_list.len());

                for (key, value) in self.candidate_list[from..to].iter() {
                    let quitted = ui
                        .horizontal(|ui| {
                            ui.colored_label(egui::Color32::LIGHT_BLUE, key);
                            ui.separator();
                            if ui.button(value).clicked() {
                                true
                            } else {
                                false
                            }
                        })
                        .inner;

                    if quitted {
                        self.stdout.write_all(key.as_bytes()).unwrap();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("candidate-footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for i in 0..self.max_page_index + 1 {
                    if i == self.page_index {
                        egui::Button::new(
                            egui::RichText::new(format!("[{}]", i + 1))
                                .color(egui::Color32::YELLOW),
                        )
                        .ui(ui);
                    } else {
                        if ui.button(format!("{}", i + 1)).clicked() {
                            self.page_index = i;
                        }
                    };
                }
            });
        });
    }
}

fn main() -> io::Result<()> {
    let mut buf = String::with_capacity(4096);
    let stdin = io::stdin();
    let stdout = io::stdout();

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
        candidate_list.push((key, value));
    }

    eframe::run_native(
        "kime-candidate",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 400.0])
                .with_decorations(false)
                .with_window_level(egui::WindowLevel::AlwaysOnTop),
            ..Default::default()
        },
        Box::new(|cc| {
            let config = kime_engine_core::load_engine_config_from_config_dir().unwrap_or_default();
            let (font_bytes, _index) = config.candidate_font;
            let mut font_data = BTreeMap::<_, Arc<egui::FontData>>::new();
            let mut families = BTreeMap::new();

            font_data.insert(
                "Font".to_string(),
                Arc::new(egui::FontData::from_owned(font_bytes.to_vec())),
            );

            families.insert(egui::FontFamily::Proportional, vec!["Font".to_string()]);
            families.insert(egui::FontFamily::Monospace, vec!["Font".to_string()]);

            cc.egui_ctx.set_fonts(egui::FontDefinitions {
                font_data,
                families,
            });

            Ok(Box::new(CandidateApp {
                stdout,
                page_index: 0,
                key_state: KeyState::default(),
                max_page_index: if candidate_list.len() % PAGE_SIZE == 0 {
                    (candidate_list.len() / PAGE_SIZE) - 1
                } else {
                    candidate_list.len() / PAGE_SIZE
                },
                candidate_list,
            }))
        }),
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}
