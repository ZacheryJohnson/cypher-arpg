use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cypher_core::affix::database::AffixDefinitionDatabase;
use cypher_core::affix::definition::{AffixDefinition, AffixDefinitionStat, AffixDefinitionTier};
use cypher_core::affix::placement::AffixPlacement;
use cypher_core::affix_pool::database::AffixPoolDefinitionDatabase;
use cypher_core::affix_pool::definition::AffixPoolDefinition;
use cypher_core::data::{DataDefinition, DataDefinitionDatabase};
use cypher_core::stat::Stat;
use cypher_item::item::database::ItemDefinitionDatabase;

use cypher_item::loot_pool::database::LootPoolDefinitionDatabase;
use eframe::egui;
use egui::{Color32, Ui};
use egui_extras::{Size, TableBuilder};

use strum::IntoEnumIterator;

fn get_affix_db_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("cypher-core");
    path.push("data");
    path.push("affix.json");
    path
}

fn get_affix_pool_db_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("cypher-core");
    path.push("data");
    path.push("affix_pool.json");
    path
}

fn get_item_db_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("cypher-item");
    path.push("data");
    path.push("item.json");
    path
}

fn get_loot_pool_db_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("cypher-item");
    path.push("data");
    path.push("loot_pool.json");
    path
}

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

struct DataEditorApp {
    affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
    affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
    item_db: Arc<Mutex<ItemDefinitionDatabase>>,
    loot_pool_db: Arc<Mutex<LootPoolDefinitionDatabase>>,

    selected_editor: SelectedEditor,
    selected_definition_id: Option<u64>,
}

impl DataEditorApp {
    fn new() -> DataEditorApp {
        let affix_db_path = get_affix_db_path();
        let affix_db = Arc::new(Mutex::new(AffixDefinitionDatabase::load_from(
            affix_db_path.to_str().unwrap(),
        )));

        let affix_pool_db_path = get_affix_pool_db_path();
        let affix_pool_db = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::load_from(
            affix_db.clone(),
            affix_pool_db_path.to_str().unwrap(),
        )));

        let item_db_path = get_item_db_path();
        let item_db = Arc::new(Mutex::new(ItemDefinitionDatabase::load_from(
            affix_pool_db.clone(),
            item_db_path.to_str().unwrap(),
        )));

        let loot_pool_db_path = get_loot_pool_db_path();
        let loot_pool_db = Arc::new(Mutex::new(LootPoolDefinitionDatabase::load_from(
            item_db.clone(),
            loot_pool_db_path.to_str().unwrap(),
        )));

        DataEditorApp {
            affix_db,
            affix_pool_db,
            item_db,
            loot_pool_db,

            selected_editor: SelectedEditor::NoEditor,
            selected_definition_id: None,
        }
    }

    /// Loads data files from the repository.
    fn load_data(&self) {
        println!("Loading data files");
    }

    /// Writes data files back to the repository.
    fn write_data(&mut self) {
        println!("Writing data files");

        // self.affix_db.lock().unwrap().write_to(get_affix_db_path());
        // self.affix_pool_db.lock().unwrap().write_to(get_affix_pool_db_path());
        // self.item_db.lock().unwrap().write_to(get_item_db_path());
        // self.loot_pool_db.lock().unwrap().write_to(get_loot_pool_db_path());
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
                let affix_db = self.affix_db.lock().unwrap();
                let mut affixes = affix_db.definitions();
                let next_id = affixes.last().unwrap().lock().unwrap().id + 1;
                let new_affix = AffixDefinition {
                    id: next_id,
                    placement: AffixPlacement::Invalid,
                    tiers: BTreeMap::new(),
                    name: String::new(),
                };

                affixes.push(Arc::new(Mutex::new(new_affix)));

                // self.invalidate_selections();
            }
        });

        if self.selected_definition_id.is_some() {
            egui::SidePanel::right("affix_right_panel")
                .min_width(300.)
                .show(ctx, |ui| {
                    if ui.button("Close").clicked() {
                        // self.invalidate_selections();
                        return;
                    }
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let affix_db = self.affix_db.lock().unwrap();
                        let mut affixes = affix_db.definitions();
                        let mut affix = affixes
                            .iter_mut()
                            .find(|def| {
                                def.lock().unwrap().id as u64
                                    == self.selected_definition_id.unwrap()
                            })
                            .unwrap()
                            .lock()
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
                    let affix_db = self.affix_db.lock().unwrap();
                    let affixes = affix_db.definitions();

                    body.rows(30., affixes.len(), |idx, mut row| {
                        let affix = affixes[idx].lock().unwrap();

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
                                    self.selected_definition_id = Some(affix.id as u64);
                                }
                            };

                        make_col_fn(&affix, affix.id.to_string(), &mut row);
                        make_col_fn(&affix, affix.name.clone(), &mut row);
                        make_col_fn(&affix, affix.placement.to_string(), &mut row);
                        make_col_fn(&affix, affix.tiers.len().to_string(), &mut row);
                    });
                });
        });
    }

    fn draw_affix_pool_editor(&mut self, ctx: &egui::Context, _ui: &mut Ui) {
        egui::TopBottomPanel::top("affix_pool_menu_bar").show(ctx, |ui| {
            if ui.button("Add").clicked() {
                let affix_pool_db = self.affix_pool_db.lock().unwrap();
                let mut affix_pools = affix_pool_db.definitions();
                let next_id = affix_pools.last().unwrap().lock().unwrap().id + 1;
                let new_affix_pool = AffixPoolDefinition {
                    id: next_id,
                    members: vec![],
                    name: String::default(),
                };

                affix_pools.push(Arc::new(Mutex::new(new_affix_pool)));

                // self.invalidate_selections();
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
                                // self.invalidate_selections();
                                should_close_sidebar = true;
                            }
                            if ui.button("Delete").clicked() {
                                let affix_pool_db = self.affix_pool_db.lock().unwrap();
                                let mut affix_pools = affix_pool_db.definitions();
                                if let Ok(idx) = affix_pools.binary_search_by(|def| {
                                    (def.lock().unwrap().id as u64)
                                        .cmp(&self.selected_definition_id.unwrap())
                                }) {
                                    affix_pools.remove(idx);
                                }

                                // self.invalidate_selections();
                                should_close_sidebar = true;
                            }
                        });

                        if should_close_sidebar {
                            return;
                        }
                    }

                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let affix_pool_db = self.affix_pool_db.lock().unwrap();
                        let mut affix_pools = affix_pool_db.definitions();
                        let affix_pool = affix_pools
                            .iter_mut()
                            .find(|def| {
                                def.lock().unwrap().id as u64
                                    == self.selected_definition_id.unwrap()
                            })
                            .unwrap();

                        ui.label(format!("Id: {}", affix_pool.lock().unwrap().id));
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            ui.text_edit_singleline(&mut affix_pool.lock().unwrap().name);
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

                        /*
                        // TODO: make DragValue work
                        for member in &mut affix_pool.members {
                            ui.horizontal(|ui| {
                                ui.label(format!("Id: {}", member.affix_def.id));
                                ui.horizontal(|ui| {
                                    ui.label("Weight");
                                    ui.add(egui::DragValue::new(&mut member.weight));
                                });
                            });
                        }
                        */
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
                    let affix_pool_db = self.affix_pool_db.lock().unwrap();
                    let affix_pools = affix_pool_db.definitions();

                    body.rows(30., affix_pools.len(), |idx, mut row| {
                        let affix_pool = affix_pools[idx].lock().unwrap();

                        let mut make_col_fn =
                            |affix_pool: &AffixPoolDefinition,
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
                                    self.selected_definition_id = Some(affix_pool.id as u64);
                                }
                            };

                        make_col_fn(&affix_pool, affix_pool.id.to_string(), &mut row);
                        make_col_fn(&affix_pool, affix_pool.name.clone(), &mut row);
                        make_col_fn(&affix_pool, affix_pool.members.len().to_string(), &mut row);
                    });
                });
        });
    }

    fn draw_item_editor(&mut self, ui: &mut Ui) {
        // ZJ-TODO: locking mutex at this wide of a scope is real dangerous
        //          I've deadlocked the others already - do it in the smallest scope possible
        let item_db = self.item_db.lock().unwrap();
        let items = item_db.definitions();

        for item in &items {
            ui.label(format!("{:?}", item));
        }
    }

    fn draw_loot_pool_editor(&mut self, ui: &mut Ui) {
        // ZJ-TODO: locking mutex at this wide of a scope is real dangerous
        //          I've deadlocked the others already - do it in the smallest scope possible
        let loot_pool_db = self.loot_pool_db.lock().unwrap();
        let loot_pools = loot_pool_db.definitions();

        for pool in &loot_pools {
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
    let app = DataEditorApp::new();
    app.load_data();

    eframe::run_native("Cypher Data Editor", options, Box::new(|_cc| Box::new(app)));
}
