use std::io::BufWriter;
use std::{collections::BTreeMap, fs::OpenOptions};

use cypher_core::affix::{
    AffixDefinition, AffixDefinitionStat, AffixDefinitionTier, AffixPlacement,
};
use cypher_core::stat::Stat;
use cypher_item::item::ItemDefinition;
use cypher_item::loot::LootPoolDefinition;

use eframe::egui;
use egui::{Color32, Ui};
use egui_extras::{Size, TableBuilder};

use strum::IntoEnumIterator;

#[derive(PartialEq)]
enum SelectedEditor {
    NoEditor,
    Affix,
    Item,
    LootPool,
}

impl Default for SelectedEditor {
    fn default() -> Self {
        SelectedEditor::NoEditor
    }
}

#[derive(Default)]
struct DataEditorApp {
    affixes: Vec<AffixDefinition>,
    items: Vec<ItemDefinition>,
    loot_pools: Vec<LootPoolDefinition>,
    selected_editor: SelectedEditor,
    selected_definition_id: Option<u32>,
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

    fn validate_data(&mut self) -> bool {
        self.affixes.iter().all(|affix| affix.validate())
    }

    /// Writes data files back to the repository.
    fn write_data(&mut self) {
        if !self.validate_data() {
            return;
        }

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

    fn draw_affix_editor(&mut self, ctx: &egui::Context, _ui: &mut Ui) {
        egui::TopBottomPanel::top("affix_menu_bar").show(ctx, |ui| {
            if ui.button("Add").clicked() {
                let next_id = self.affixes.last().unwrap().id + 1;
                let new_affix = AffixDefinition {
                    id: next_id,
                    placement: AffixPlacement::Invalid,
                    tiers: BTreeMap::new(),
                };

                self.affixes.push(new_affix);

                self.invalidate_selections();
            }
        });

        if self.selected_definition_id.is_some() {
            egui::SidePanel::right("affix_right_panel")
                .min_width(300.)
                .show(ctx, |ui| {
                    if ui.button("Close").clicked() {
                        self.invalidate_selections();
                        return;
                    }
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let affix_idx = self
                            .affixes
                            .binary_search_by(|probe| {
                                probe.id.cmp(&self.selected_definition_id.unwrap())
                            })
                            .unwrap();
                        let affix = &mut self.affixes[affix_idx];
                        ui.label(format!("Id: {}", affix.id));

                        let selected_placement = &mut affix.placement;
                        egui::ComboBox::from_label("Placement")
                            .selected_text(format!("{:?}", selected_placement))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    selected_placement,
                                    AffixPlacement::Prefix,
                                    "Prefix",
                                );
                                ui.selectable_value(
                                    selected_placement,
                                    AffixPlacement::Suffix,
                                    "Suffix",
                                );
                            });

                        ui.label("Tiers");

                        ui.horizontal(|ui| {
                            if ui.button("Add").clicked() {
                                let next_id = affix.tiers.len() as u16 + 1;
                                affix.tiers.insert(
                                    next_id,
                                    AffixDefinitionTier {
                                        tier: next_id, // TODO: fix this...
                                        stats: Vec::new(),
                                        item_level_req: None,
                                        precision_places: None,
                                    },
                                );
                            }

                            if ui.button("Remove").clicked() {
                                let last_id = affix.tiers.len() as u16;
                                affix.tiers.remove(&last_id);
                            }
                        });

                        for (tier_id, tier_def) in &mut affix.tiers {
                            ui.label(format!("T{}", tier_id));
                            // TODO: extract to draw_affix_definition_tier
                            ui.horizontal(|ui| {
                                if ui.button("Add Stat").clicked() {
                                    tier_def.stats.push(AffixDefinitionStat {
                                        stat: Stat::Resolve, // TODO: invalid?
                                        lower_bound: 0.,
                                        upper_bound: 0.,
                                    });
                                }

                                if ui.button("Remove Stat").clicked() {
                                    tier_def.stats.remove(tier_def.stats.len() - 1);
                                }
                            });
                            for stat in &mut tier_def.stats {
                                let selected_stat = &mut stat.stat;
                                egui::ComboBox::from_id_source(format!(
                                    "Stat_{}-{}",
                                    stat.lower_bound, stat.upper_bound
                                ))
                                .selected_text(format!("{:?}", selected_stat))
                                .show_ui(ui, |ui| {
                                    for stat_variant in Stat::iter() {
                                        ui.selectable_value(
                                            selected_stat,
                                            stat_variant,
                                            format!("{:?}", stat_variant),
                                        );
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut stat.lower_bound,
                                            (stat.upper_bound - 100_f32)..=stat.upper_bound,
                                        )
                                        .text("Lower Bound"),
                                    );
                                    ui.add(
                                        egui::Slider::new(
                                            &mut stat.upper_bound,
                                            stat.lower_bound..=(stat.lower_bound + 100_f32),
                                        )
                                        .text("Upper Bound"),
                                    );
                                });
                            }
                            ui.label(format!(
                                "Level Req: {}",
                                tier_def.item_level_req.unwrap_or(0)
                            ));
                            ui.label(format!(
                                "Float Precision: {}",
                                tier_def.precision_places.unwrap_or(0)
                            ));
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
                .column(Size::initial(60.0).at_least(40.0))
                .column(Size::initial(60.0).at_least(40.0))
                .column(Size::remainder().at_least(60.0))
                .resizable(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Id");
                    });
                    header.col(|ui| {
                        ui.heading("Placement");
                    });
                    header.col(|ui| {
                        ui.heading("Tiers");
                    });
                })
                .body(|body| {
                    body.rows(30., self.affixes.len(), |idx, mut row| {
                        let affix = &self.affixes[idx];

                        let mut make_col_fn =
                            |affix: &AffixDefinition,
                             text: String,
                             row: &mut egui_extras::TableRow| {
                                let x = row.col(|ui| {
                                    if affix.validate() {
                                        ui.label(text);
                                    } else {
                                        ui.colored_label(Color32::RED, text);
                                    }
                                });

                                if x.interact(egui::Sense::click()).clicked() {
                                    self.selected_definition_id = Some(affix.id);
                                }
                            };

                        make_col_fn(affix, affix.id.to_string(), &mut row);
                        make_col_fn(affix, affix.placement.to_string(), &mut row);
                        make_col_fn(affix, affix.tiers.len().to_string(), &mut row);
                    });
                });
        });
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

    fn invalidate_selections(&mut self) {
        self.selected_definition_id = None;
    }
}

impl eframe::App for DataEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.menu_button("File", |ui| self.draw_file_menu_options(ui));
                ui.separator();

                if ui.button("Affix").clicked() {
                    self.selected_editor = SelectedEditor::Affix;
                    self.invalidate_selections();
                }
                if ui.button("Item").clicked() {
                    self.selected_editor = SelectedEditor::Item;
                    self.invalidate_selections();
                }
                if ui.button("LootPool").clicked() {
                    self.selected_editor = SelectedEditor::LootPool;
                    self.invalidate_selections();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_editor {
            SelectedEditor::Affix => self.draw_affix_editor(ctx, ui),
            SelectedEditor::Item => self.draw_item_editor(ui),
            SelectedEditor::LootPool => self.draw_loot_pool_editor(ui),
            _ => {}
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let mut app = DataEditorApp::default();
    app.load_data();

    eframe::run_native("Data Editor", options, Box::new(|_cc| Box::new(app)));
}
