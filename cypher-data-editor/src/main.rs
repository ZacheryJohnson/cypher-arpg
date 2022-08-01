use std::io::BufWriter;
use std::{collections::BTreeMap, fs::OpenOptions};

use cypher_core::affix::definition::{AffixDefinition, AffixDefinitionStat, AffixDefinitionTier};
use cypher_core::affix::placement::AffixPlacement;
use cypher_core::affix::pool::AffixPoolDefinition;
use cypher_core::data::DataDefinition;
use cypher_core::stat::Stat;
use cypher_item::item::ItemDefinition;
use cypher_item::loot_pool::LootPoolDefinition;

use eframe::egui;
use egui::{Color32, Ui};
use egui_extras::{Size, TableBuilder};

use serde::Serialize;
use strum::IntoEnumIterator;

#[derive(Debug, PartialEq)]
enum SelectedEditor {
    NoEditor,
    Affix,
    AffixPool,
    Item,
    LootPool,
}

impl Default for SelectedEditor {
    fn default() -> Self {
        SelectedEditor::NoEditor
    }
}

#[derive(Default)]
struct DataEditorApp<'db> {
    affixes: Vec<AffixDefinition>,
    affix_pools: Vec<AffixPoolDefinition<'db>>,
    items: Vec<ItemDefinition<'db>>,
    loot_pools: Vec<LootPoolDefinition<'db>>,
    selected_editor: SelectedEditor,
    selected_definition_id: Option<u32>,
}

impl<'db> DataEditorApp<'db> {
    /// Loads data files from the repository.
    fn load_data(&mut self) {
        println!("Loading data files");

        todo!("Initialize databases")
    }

    fn validate_data(&mut self) -> bool {
        self.affixes.iter().all(|affix| affix.validate())
            && self
                .affix_pools
                .iter()
                .all(|affix_pool| affix_pool.validate())
            && self.items.iter().all(|item| item.validate())
            && self.loot_pools.iter().all(|loot_pool| loot_pool.validate())
    }

    /// Writes data files back to the repository.
    fn write_data(&mut self) {
        if !self.validate_data() {
            println!("Data failed validation - failing the save.");
            return;
        }

        println!("Writing data files");

        fn write_to_file<T>(path: &'static str, data: &T)
        where
            T: Serialize,
        {
            let root_dir =
                String::from(project_root::get_project_root().unwrap().to_str().unwrap());

            let mut file_open_options = OpenOptions::new();
            file_open_options.write(true).append(false).truncate(true);

            let file = file_open_options
                .clone()
                .open(std::path::Path::new(&root_dir).join(path))
                .unwrap();
            let bufwriter = BufWriter::new(file);
            serde_json::ser::to_writer_pretty(bufwriter, data).unwrap();
        }

        write_to_file("cypher-core/data/affix.json", &self.affixes);
        write_to_file("cypher-core/data/affix_pool.json", &self.affix_pools);
        write_to_file("cypher-item/data/item.json", &self.items);
        write_to_file("cypher-item/data/loot_pool.json", &self.loot_pools);
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
                    name: String::new(),
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
                        let affix = self
                            .affixes
                            .iter_mut()
                            .find(|def| def.id == self.selected_definition_id.unwrap())
                            .unwrap();

                        ui.label(format!("Id: {}", affix.id));
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            ui.text_edit_singleline(&mut affix.name);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Placement");

                            let selected_placement = &mut affix.placement;
                            egui::ComboBox::from_id_source("Placement")
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
                        });

                        ui.separator();
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
                            ui.collapsing(format!("T{}", tier_id), |ui| {
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
                                            .text("Lower Bound")
                                            .fixed_decimals(
                                                tier_def.precision_places.unwrap_or(0) as usize
                                            ),
                                        );
                                        ui.add(
                                            egui::Slider::new(
                                                &mut stat.upper_bound,
                                                stat.lower_bound..=(stat.lower_bound + 100_f32),
                                            )
                                            .text("Upper Bound")
                                            .fixed_decimals(
                                                tier_def.precision_places.unwrap_or(0) as usize
                                            ),
                                        );
                                    });
                                }
                                ui.horizontal(|ui| {
                                    let mut enabled = tier_def.item_level_req.is_some();
                                    ui.checkbox(&mut enabled, "Level Req");
                                    if enabled {
                                        if tier_def.item_level_req.is_none() {
                                            tier_def.item_level_req = Some(0);
                                        }
                                        let mut val = tier_def.item_level_req.unwrap();
                                        ui.add(egui::Slider::new(
                                            &mut val,
                                            0..=100, // TODO: min + max level definition
                                        ));
                                        tier_def.item_level_req = Some(val);
                                    } else if tier_def.item_level_req.is_some() {
                                        tier_def.item_level_req = None;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    let mut enabled = tier_def.precision_places.is_some();
                                    ui.checkbox(&mut enabled, "Float Precision");
                                    if enabled {
                                        if tier_def.precision_places.is_none() {
                                            tier_def.precision_places = Some(0);
                                        }
                                        let mut val = tier_def.precision_places.unwrap();
                                        ui.add(egui::Slider::new(&mut val, 1..=5));
                                        tier_def.precision_places = Some(val);
                                    } else if tier_def.precision_places.is_some() {
                                        tier_def.precision_places = None;
                                    }
                                });
                            });
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
                .column(Size::initial(60.0).at_least(40.0))
                .column(Size::remainder().at_least(60.0))
                .resizable(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Id");
                    });
                    header.col(|ui| {
                        ui.heading("Name");
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
                        make_col_fn(affix, affix.name.clone(), &mut row);
                        make_col_fn(affix, affix.placement.to_string(), &mut row);
                        make_col_fn(affix, affix.tiers.len().to_string(), &mut row);
                    });
                });
        });
    }

    fn draw_affix_pool_editor(&mut self, ctx: &egui::Context, _ui: &mut Ui) {
        egui::TopBottomPanel::top("affix_pool_menu_bar").show(ctx, |ui| {
            if ui.button("Add").clicked() {
                let next_id = self.affix_pools.last().unwrap().id + 1;
                let new_affix_pool = AffixPoolDefinition {
                    id: next_id,
                    members: vec![],
                    name: String::default(),
                };

                self.affix_pools.push(new_affix_pool);

                self.invalidate_selections();
            }
        });

        if self.selected_definition_id.is_some() {
            egui::SidePanel::right("affix_pool_right_panel")
                .min_width(300.)
                .show(ctx, |ui| {
                    {
                        let mut should_close_sidebar = false;
                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() {
                                self.invalidate_selections();
                                should_close_sidebar = true;
                            }
                            if ui.button("Delete").clicked() {
                                if let Ok(idx) = self.affix_pools.binary_search_by(|def| {
                                    def.id.cmp(&self.selected_definition_id.unwrap())
                                }) {
                                    self.affix_pools.remove(idx);
                                }

                                self.invalidate_selections();
                                should_close_sidebar = true;
                            }
                        });

                        if should_close_sidebar {
                            return;
                        }
                    }

                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let affix_pool = self
                            .affix_pools
                            .iter_mut()
                            .find(|def| def.id == self.selected_definition_id.unwrap())
                            .unwrap();

                        ui.label(format!("Id: {}", affix_pool.id));
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            ui.text_edit_singleline(&mut affix_pool.name);
                        });

                        ui.separator();
                        ui.label("Members");
                        if ui.button("Add").clicked() {
                            /*
                            affix_pool.members.push(AffixPoolMember {
                                affix_id: 0,
                                weight: 0,
                            });
                            */
                            todo!("Implement adding new affix pool members")
                        }

                        for member in &mut affix_pool.members {
                            ui.horizontal(|ui| {
                                ui.label(format!("Id: {}", member.affix_def.id));
                                ui.horizontal(|ui| {
                                    ui.label("Weight");
                                    ui.add(egui::DragValue::new(&mut member.weight));
                                });
                            });
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
                .column(Size::initial(60.0).at_least(40.0))
                .column(Size::remainder().at_least(60.0))
                .resizable(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Id");
                    });
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Members");
                    });
                })
                .body(|body| {
                    body.rows(30., self.affix_pools.len(), |idx, mut row| {
                        let affix_pool = &self.affix_pools[idx];

                        let mut make_col_fn =
                            |affix: &AffixPoolDefinition,
                             text: String,
                             row: &mut egui_extras::TableRow| {
                                let x = row.col(|ui| {
                                    if affix_pool.validate() {
                                        ui.label(text);
                                    } else {
                                        ui.colored_label(Color32::RED, text);
                                    }
                                });

                                if x.interact(egui::Sense::click()).clicked() {
                                    self.selected_definition_id = Some(affix.id);
                                }
                            };

                        make_col_fn(affix_pool, affix_pool.id.to_string(), &mut row);
                        make_col_fn(affix_pool, affix_pool.name.clone(), &mut row);
                        make_col_fn(affix_pool, affix_pool.members.len().to_string(), &mut row);
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

impl<'db> eframe::App for DataEditorApp<'db> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.menu_button("File", |ui| self.draw_file_menu_options(ui));
                ui.separator();

                ui.menu_button("Affix", |ui| {
                    if ui.button("Definitions").clicked() {
                        self.selected_editor = SelectedEditor::Affix;
                        self.invalidate_selections();
                        ui.close_menu();
                    }

                    if ui.button("Pools").clicked() {
                        self.selected_editor = SelectedEditor::AffixPool;
                        self.invalidate_selections();
                        ui.close_menu();
                    }
                });

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
            SelectedEditor::AffixPool => self.draw_affix_pool_editor(ctx, ui),
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

    eframe::run_native("Cypher Data Editor", options, Box::new(|_cc| Box::new(app)));
}
