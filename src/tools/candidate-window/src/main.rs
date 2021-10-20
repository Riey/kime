use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{self, BufRead, Stdout, Write},
};

const PAGE_SIZE: usize = 10;

struct CandidateApp {
    stdout: Stdout,
    page_index: usize,
    max_page_index: usize,
    candidate_list: Vec<(String, String)>,
}

impl eframe::epi::App for CandidateApp {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        // egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
        //     ui.heading("Candiate");
        // });
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
                        frame.quit();
                        return;
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("candidate-footer").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("<").clicked() {
                        self.page_index = self.page_index.saturating_sub(1);
                    }

                    ui.label(format!("page {}", self.page_index + 1));

                    if ui.button(">").clicked() {
                        self.page_index =
                            self.page_index.saturating_add(1).min(self.max_page_index);
                    }
                });
            });
        });
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        let mut font_data = BTreeMap::<_, Cow<'static, [u8]>>::new();
        let mut fonts_for_family = BTreeMap::new();

        font_data.insert(
            "NanumBarunGothic".to_string(),
            Cow::Borrowed(include_bytes!("/nix/store/imnk1n6llkh089xgzqyqpr6yw9qz9b3z-d2codingfont-1.3.2/share/fonts/truetype/D2Coding-Ver1.3.2-20180524-all.ttc")),
        );
        font_data.insert(
            "NanumGothicCoding".to_string(),
            Cow::Borrowed(include_bytes!("/nix/store/imnk1n6llkh089xgzqyqpr6yw9qz9b3z-d2codingfont-1.3.2/share/fonts/truetype/D2Coding-Ver1.3.2-20180524-all.ttc")),
        );

        fonts_for_family.insert(
            egui::FontFamily::Proportional,
            vec!["NanumBarunGothic".to_string()],
        );
        fonts_for_family.insert(
            egui::FontFamily::Monospace,
            vec!["NanumGothicCoding".to_string()],
        );

        let mut family_and_size = BTreeMap::new();
        family_and_size.insert(
            egui::TextStyle::Small,
            (egui::FontFamily::Proportional, 18.0),
        );
        family_and_size.insert(
            egui::TextStyle::Body,
            (egui::FontFamily::Proportional, 22.0),
        );
        family_and_size.insert(
            egui::TextStyle::Button,
            (egui::FontFamily::Proportional, 24.0),
        );
        family_and_size.insert(
            egui::TextStyle::Heading,
            (egui::FontFamily::Proportional, 30.0),
        );
        family_and_size.insert(
            egui::TextStyle::Monospace,
            (egui::FontFamily::Monospace, 22.0),
        );

        ctx.set_fonts(egui::FontDefinitions {
            font_data,
            fonts_for_family,
            family_and_size,
        });
    }

    fn name(&self) -> &str {
        "kime-candidate"
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
        Box::new(CandidateApp {
            stdout,
            page_index: 0,
            max_page_index: if candidate_list.len() % PAGE_SIZE == 0 {
                (candidate_list.len() / PAGE_SIZE) - 1
            } else {
                candidate_list.len() / PAGE_SIZE
            },
            candidate_list,
        }),
        eframe::NativeOptions {
            always_on_top: true,
            decorated: false,
            drag_and_drop_support: false,
            icon_data: None,
            initial_window_size: Some(egui::vec2(400.0, 400.0)),
            resizable: false,
            transparent: false,
        },
    );

    Ok(())
}
