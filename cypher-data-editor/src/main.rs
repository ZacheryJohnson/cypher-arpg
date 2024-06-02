mod item;
mod loot_pool;
mod table_display;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cypher_core::affix::database::AffixDefinitionDatabase;
use cypher_core::affix::definition::{
    AffixDefinition, AffixDefinitionStat, AffixDefinitionTier, AffixDefinitionValue,
};
use cypher_core::affix::placement::AffixPlacement;
use cypher_core::affix_pool::database::AffixPoolDefinitionDatabase;
use cypher_core::affix_pool::definition::AffixPoolDefinition;
use cypher_core::affix_pool::member::AffixPoolMember;
use cypher_core::data::{DataDefinition, DataDefinitionDatabase};
use cypher_core::stat::Stat;
use cypher_item::item::classification::{ItemClassification, ItemEquipSlot};
use cypher_item::item::database::ItemDefinitionDatabase;

use cypher_item::item::definition::ItemDefinition;
use cypher_item::loot_pool::database::LootPoolDefinitionDatabase;
use cypher_item::loot_pool::definition::LootPoolDefinition;
use eframe::egui;
use egui::{Color32, Ui, WidgetText};
use egui_extras::{Column, TableBuilder};

use crate::table_display::TableDisplay;
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
            &(),
        )));

        let affix_pool_db_path = get_affix_pool_db_path();
        let affix_pool_db = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::load_from(
            affix_pool_db_path.to_str().unwrap(),
            &affix_db.clone(),
        )));

        let item_db_path = get_item_db_path();
        let item_db = Arc::new(Mutex::new(ItemDefinitionDatabase::load_from(
            item_db_path.to_str().unwrap(),
            &(affix_db.clone(), affix_pool_db.clone()),
        )));

        let loot_pool_db_path = get_loot_pool_db_path();
        let loot_pool_db = Arc::new(Mutex::new(LootPoolDefinitionDatabase::load_from(
            loot_pool_db_path.to_str().unwrap(),
            &item_db.clone(),
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

        self.affix_db
            .lock()
            .unwrap()
            .write_to(get_affix_db_path().to_str().unwrap());
        self.affix_pool_db
            .lock()
            .unwrap()
            .write_to(get_affix_pool_db_path().to_str().unwrap());
        self.item_db
            .lock()
            .unwrap()
            .write_to(get_item_db_path().to_str().unwrap());
        self.loot_pool_db
            .lock()
            .unwrap()
            .write_to(get_loot_pool_db_path().to_str().unwrap());
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

    fn populate_row_for_definition<T>(&mut self, def: &T, row: &mut egui_extras::TableRow)
    where
        T: TableDisplay + DataDefinition,
    {
        for text_value in def.data_row_values() {
            let (_, resp) = row.col(|ui| {
                if def.validate() {
                    let label = egui::Label::new(text_value).selectable(false);
                    ui.add(label);
                } else {
                    let widget_text = WidgetText::from(text_value).color(Color32::RED);
                    let label = egui::Label::new(widget_text).selectable(false);
                    ui.add(label);
                }
            });

            if resp.interact(egui::Sense::click()).clicked() {
                self.selected_definition_id = Some(def.id() as u64);
            }
        }
    }

    fn draw_affix_editor(&mut self, ctx: &egui::Context, _ui: &mut Ui) {
        egui::TopBottomPanel::top("affix_menu_bar").show(ctx, |ui| {
            if ui.button("Add").clicked() {
                self.invalidate_selections();

                let mut affix_db = self.affix_db.lock().unwrap();
                let affixes = affix_db.definitions();
                let next_id = affixes.iter().fold(0, |acc, next| {
                    if acc > next.lock().unwrap().id {
                        acc
                    } else {
                        next.lock().unwrap().id
                    }
                }) + 1;
                let new_affix = AffixDefinition {
                    id: next_id,
                    placement: AffixPlacement::Invalid,
                    tiers: BTreeMap::new(),
                    name: String::new(),
                };

                affix_db.add_definition(new_affix);
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
                                            value: AffixDefinitionValue::Range(0., 0.),
                                        });
                                    }

                                    if ui.button("Remove Stat").clicked() {
                                        tier_def.stats.remove(tier_def.stats.len() - 1);
                                    }
                                });
                                for stat in &mut tier_def.stats {
                                    let selected_stat = &mut stat.stat;
                                    egui::ComboBox::from_id_source(format!(
                                        "Stat_{:?}_{}",
                                        selected_stat, stat.value
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
                                        match &mut stat.value {
                                            AffixDefinitionValue::Exact(val) => {
                                                ui.add(
                                                    egui::Slider::new(
                                                        val,
                                                        (*val - 50_f32)..=(*val + 50_f32),
                                                    )
                                                    .text("Lower Bound")
                                                    .fixed_decimals(
                                                        tier_def.precision_places.unwrap_or(0)
                                                            as usize,
                                                    ),
                                                );
                                            }
                                            AffixDefinitionValue::Range(lower, upper) => {
                                                ui.add(
                                                    egui::Slider::new(
                                                        lower,
                                                        (*upper - 100_f32)..=*upper,
                                                    )
                                                    .text("Lower Bound")
                                                    .fixed_decimals(
                                                        tier_def.precision_places.unwrap_or(0)
                                                            as usize,
                                                    ),
                                                );
                                                ui.add(
                                                    egui::Slider::new(
                                                        upper,
                                                        *lower..=(*lower + 100_f32),
                                                    )
                                                    .text("Upper Bound")
                                                    .fixed_decimals(
                                                        tier_def.precision_places.unwrap_or(0)
                                                            as usize,
                                                    ),
                                                );
                                            }
                                        };
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
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(60.0).at_least(40.0))
                .column(Column::initial(60.0).at_least(40.0))
                .column(Column::initial(60.0).at_least(40.0))
                .column(Column::remainder().at_least(60.0))
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
                    let mut affixes = {
                        let affix_db = self.affix_db.lock().unwrap();
                        affix_db.definitions()
                    };

                    // ZJ-TODO: other sorting/filtering methods? would be nice to sort/filter other columns, not just ID
                    affixes.sort_by(|a, b| a.lock().unwrap().id.cmp(&b.lock().unwrap().id));

                    body.rows(30., affixes.len(), |mut row| {
                        let affix = affixes[row.index()].lock().unwrap();

                        if let Some(selected_definition_id) = self.selected_definition_id {
                            row.set_selected(selected_definition_id == affix.id as u64);
                        }

                        self.populate_row_for_definition(&*affix, &mut row);
                    });
                });
        });
    }

    fn draw_affix_pool_editor(&mut self, ctx: &egui::Context, _ui: &mut Ui) {
        egui::TopBottomPanel::top("affix_pool_menu_bar").show(ctx, |ui| {
            if ui.button("Add").clicked() {
                self.invalidate_selections();

                let mut affix_pool_db = self.affix_pool_db.lock().unwrap();
                let affix_pools = affix_pool_db.definitions();
                let next_id = affix_pools.last().unwrap().lock().unwrap().id + 1;
                let new_affix_pool = AffixPoolDefinition {
                    id: next_id,
                    members: vec![],
                    name: String::default(),
                };

                affix_pool_db.add_definition(new_affix_pool);
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
                                {
                                    // ZJ-TODO: deletions
                                    let affix_pool_db = self.affix_pool_db.lock().unwrap();
                                    let mut affix_pools = affix_pool_db.definitions();
                                    if let Ok(idx) = affix_pools.binary_search_by(|def| {
                                        (def.lock().unwrap().id as u64)
                                            .cmp(&self.selected_definition_id.unwrap())
                                    }) {
                                        affix_pools.remove(idx);
                                    }

                                    should_close_sidebar = true;
                                }

                                self.invalidate_selections();
                            }
                        });

                        if should_close_sidebar {
                            return;
                        }
                    }

                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let affix_pool_db = self.affix_pool_db.lock().unwrap();
                        let affix_pools = affix_pool_db.definitions();
                        let mut affix_pool = affix_pools
                            .iter()
                            .find(|def| {
                                def.lock().unwrap().id as u64
                                    == self.selected_definition_id.unwrap()
                            })
                            .unwrap()
                            .lock()
                            .unwrap();

                        ui.label(format!("Id: {}", affix_pool.id));
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            ui.text_edit_singleline(&mut affix_pool.name);
                        });

                        ui.separator();
                        ui.label("Members");
                        if ui.button("Add").clicked() {
                            let definition = AffixPoolMember {
                                affix_def: self
                                    .affix_db
                                    .lock()
                                    .unwrap()
                                    .definitions()
                                    .first()
                                    .unwrap()
                                    .to_owned(),
                                weight: 0,
                            };
                            affix_pool.members.push(definition);
                        }

                        for member in &mut affix_pool.members {
                            let member_def = member.affix_def.lock().unwrap();
                            ui.horizontal(|ui| {
                                ui.label(format!("{}", member_def.name));
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
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(60.0).at_least(40.0))
                .column(Column::initial(60.0).at_least(40.0))
                .column(Column::initial(60.0).at_least(40.0))
                .column(Column::remainder().at_least(60.0))
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
                    let affix_pools = {
                        let affix_pool_db = self.affix_pool_db.lock().unwrap();
                        affix_pool_db.definitions()
                    };

                    body.rows(30., affix_pools.len(), |mut row| {
                        let affix_pool = affix_pools[row.index()].lock().unwrap();

                        if let Some(selected_definition_id) = self.selected_definition_id {
                            row.set_selected(selected_definition_id == affix_pool.id as u64);
                        }

                        self.populate_row_for_definition(&*affix_pool, &mut row)
                    });
                });
        });
    }

    fn draw_item_editor(&mut self, ctx: &egui::Context, _ui: &mut Ui) {
        item::draw(ctx, self);
    }

    fn draw_loot_pool_editor(&mut self, ctx: &egui::Context, _ui: &mut Ui) {
        loot_pool::draw(ctx, self);
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
            SelectedEditor::Item => self.draw_item_editor(ctx, ui),
            SelectedEditor::LootPool => self.draw_loot_pool_editor(ctx, ui),
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
