use std::fs::OpenOptions;
use std::io::BufWriter;

use cypher_core::affix::AffixDefinition;
use cypher_item::item::ItemDefinition;
use cypher_item::loot::LootPoolDefinition;

use eframe::egui;
use egui::Ui;

#[derive(Default)]
struct DataEditorApp {
    affixes: Vec<AffixDefinition>,
    items: Vec<ItemDefinition>,
    loot_pools: Vec<LootPoolDefinition>,
}

impl DataEditorApp {
    /// Loads data files from the repository.
    fn load_data(&mut self) {
        println!("Loading data files");

        let affix_data = include_str!("../../cypher-core/data/affix.json");
        self.affixes = serde_json::de::from_str(affix_data).unwrap();

        let item_data = include_str!("../../cypher-item/data/item.json");
        self.items = serde_json::de::from_str(item_data).unwrap();

        let loot_pool_data = include_str!("../../cypher-item/data/loot_pool.json");
        self.loot_pools = serde_json::de::from_str(loot_pool_data).unwrap();
    }

    /// Writes data files back to the repository.
    fn write_data(&mut self) {
        println!("Writing data files");

        let root_dir = String::from(project_root::get_project_root().unwrap().to_str().unwrap());

        let mut file_open_options = OpenOptions::new();
        file_open_options.write(true).append(false).truncate(true);

        let affix_file = file_open_options
            .clone()
            .open(std::path::Path::new(&root_dir).join("cypher-core/data/affix.json"))
            .unwrap();
        let affix_bufwriter = BufWriter::new(affix_file);
        serde_json::ser::to_writer_pretty(affix_bufwriter, &self.affixes).unwrap();

        let item_file = file_open_options
            .clone()
            .open(std::path::Path::new(&root_dir).join("cypher-item/data/item.json"))
            .unwrap();
        let item_bufwriter = BufWriter::new(item_file);
        serde_json::ser::to_writer_pretty(item_bufwriter, &self.affixes).unwrap();

        let loot_pool_file = file_open_options
            .clone()
            .open(std::path::Path::new(&root_dir).join("cypher-item/data/loot_pool.json"))
            .unwrap();
        let loot_pool_bufwriter = BufWriter::new(loot_pool_file);
        serde_json::ser::to_writer_pretty(loot_pool_bufwriter, &self.loot_pools).unwrap();
    }

    fn draw_file_menu_options(&mut self, ui: &mut Ui) {
        if ui.button("Reload").clicked() {
            self.load_data();
            ui.close_menu();
        };

        if ui.button("Save").clicked() {
            self.write_data();
            ui.close_menu();
        };
    }

    fn draw_affix_editor(&mut self, ui: &mut Ui) {
        for affix in &self.affixes {
            ui.label(format!("{:?}", affix));
        }
    }

    fn draw_item_editor(&mut self, ui: &mut Ui) {
        for item in &self.items {
            ui.label(format!("{:?}", item));
        }
    }

    fn draw_loot_pool_editor(&mut self, ui: &mut Ui) {
        for pool in &self.loot_pools {
            ui.label(format!("{:?}", pool));
        }
    }
}

impl eframe::App for DataEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.menu_button("File", |ui| self.draw_file_menu_options(ui));
                    ui.separator();
                    if ui.button("Affixes").clicked() {
                        self.draw_affix_editor(ui);
                    }

                    if ui.button("Items").clicked() {
                        self.draw_item_editor(ui);
                    }

                    if ui.button("Loot Pools").clicked() {
                        self.draw_loot_pool_editor(ui);
                    }
                });
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let mut app = DataEditorApp::default();
    app.load_data();

    eframe::run_native("Data Editor", options, Box::new(|_cc| Box::new(app)));
}
