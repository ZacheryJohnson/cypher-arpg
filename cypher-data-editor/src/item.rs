use crate::DataEditorApp;
use cypher_core::affix::definition::AffixDefinition;
use cypher_core::affix::placement::AffixPlacement;
use cypher_core::affix_pool::definition::AffixPoolDefinition;
use cypher_core::data::DataDefinitionDatabase;
use cypher_item::item::classification::{ItemClassification, ItemEquipSlot};
use cypher_item::item::definition::ItemDefinition;
use egui_extras::{Column, TableBuilder};
use std::sync::{Arc, Mutex};

pub fn draw(ctx: &egui::Context, app: &mut DataEditorApp) {
    egui::TopBottomPanel::top("item_menu_bar").show(ctx, |ui| {
        if ui.button("Add").clicked() {
            app.invalidate_selections();

            let mut item_db = app.item_db.lock().unwrap();
            let items = item_db.definitions();
            let next_id = items.iter().fold(0, |acc, next| {
                if acc > next.lock().unwrap().id {
                    acc
                } else {
                    next.lock().unwrap().id
                }
            }) + 1;
            let new_affix = ItemDefinition {
                id: next_id,
                classification: ItemClassification::Invalid,
                affix_pools: vec![],
                fixed_affixes: vec![],
                name: String::new(),
            };

            item_db.add_definition(new_affix);
        }
    });

    if app.selected_definition_id.is_some() {
        egui::SidePanel::right("item_right_panel")
            .min_width(300.)
            .show(ctx, |ui| {
                if ui.button("Close").clicked() {
                    app.invalidate_selections();
                    return;
                }
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let item_db = app.item_db.lock().unwrap();
                    let mut items = item_db.definitions();
                    let mut item = items
                        .iter_mut()
                        .find(|def| {
                            def.lock().unwrap().id as u64 == app.selected_definition_id.unwrap()
                        })
                        .unwrap()
                        .lock()
                        .unwrap();

                    ui.label(format!("Id: {}", item.id));
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut item.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Classification");
                        let selected_classification = &mut item.classification;
                        egui::ComboBox::from_id_source("Classification")
                            .selected_text(format!("{:?}", selected_classification))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    selected_classification,
                                    ItemClassification::Equippable(ItemEquipSlot::Head),
                                    "Head",
                                );
                                ui.selectable_value(
                                    selected_classification,
                                    ItemClassification::Equippable(ItemEquipSlot::LeftArm),
                                    "LeftArm",
                                );
                                ui.selectable_value(
                                    selected_classification,
                                    ItemClassification::Equippable(ItemEquipSlot::RightArm),
                                    "RightArm",
                                );
                                ui.selectable_value(
                                    selected_classification,
                                    ItemClassification::Equippable(ItemEquipSlot::Body),
                                    "Body",
                                );
                                ui.selectable_value(
                                    selected_classification,
                                    ItemClassification::Equippable(ItemEquipSlot::Belt),
                                    "Belt",
                                );
                                ui.selectable_value(
                                    selected_classification,
                                    ItemClassification::Equippable(ItemEquipSlot::Legs),
                                    "Legs",
                                );
                                ui.selectable_value(
                                    selected_classification,
                                    ItemClassification::Equippable(ItemEquipSlot::Boots),
                                    "Boots",
                                );
                                ui.selectable_value(
                                    selected_classification,
                                    ItemClassification::Currency,
                                    "Currency",
                                );
                            });
                    });

                    ui.separator();

                    {
                        ui.horizontal(|ui| {
                            ui.label("Fixed Affixes");
                            if ui.button("Add").clicked() {
                                let next_id = item.fixed_affixes.len() as u32;
                                item.fixed_affixes.insert(
                                    next_id as usize,
                                    Arc::new(Mutex::new(AffixDefinition {
                                        id: next_id,
                                        placement: AffixPlacement::Invalid,
                                        tiers: Default::default(),
                                        name: String::default(),
                                    })),
                                );
                            }
                        });

                        let mut remove_id = Option::None;
                        let mut replace_ids: Option<(u32, u32)> = Option::None;

                        for def in &item.fixed_affixes {
                            ui.horizontal(|ui| {
                                let (mut affix_id, affix_name) = {
                                    let affix_def = def.lock().unwrap();

                                    (affix_def.id, affix_def.name.clone())
                                };

                                egui::ComboBox::from_id_source(format!("ItemFixedAffix{:?}", def))
                                    .selected_text(format!("{}", affix_name))
                                    .show_ui(ui, |ui| {
                                        let selected_affix_id = &mut affix_id;
                                        let mut affix_definitions =
                                            app.affix_db.lock().unwrap().definitions();
                                        affix_definitions.retain(|def| {
                                            def.lock().unwrap().id != *selected_affix_id
                                        });

                                        for affix in affix_definitions {
                                            let affix_def = affix.lock().unwrap();
                                            ui.selectable_value(
                                                selected_affix_id,
                                                affix_def.id,
                                                affix_def.name.as_str(),
                                            );
                                        }
                                    });

                                if ui.button("Remove").clicked() {
                                    remove_id = Some(item.fixed_affixes.len() as u32 - 1);
                                }

                                let old_id = def.lock().unwrap().id;
                                if old_id != affix_id {
                                    replace_ids = Some((old_id, affix_id));
                                }
                            });
                        }

                        if let Some(id) = remove_id {
                            item.fixed_affixes
                                .retain(|affix| affix.lock().unwrap().id != id);
                        }

                        if let Some((old_id, new_id)) = replace_ids {
                            let item_idx = item
                                .fixed_affixes
                                .binary_search_by(|probe| probe.lock().unwrap().id.cmp(&old_id))
                                .unwrap();
                            item.fixed_affixes.remove(item_idx);
                            let new_def = app.affix_db.lock().unwrap().definition(new_id).unwrap();
                            item.fixed_affixes.push(new_def);
                        }
                    }

                    ui.separator();

                    {
                        ui.horizontal(|ui| {
                            ui.label("Affix Pools");
                            if ui.button("Add").clicked() {
                                let next_id = item.affix_pools.len() as u32;
                                item.affix_pools.insert(
                                    next_id as usize,
                                    Arc::new(Mutex::new(AffixPoolDefinition {
                                        id: next_id,
                                        members: vec![],
                                        name: String::default(),
                                    })),
                                );
                            }
                        });

                        let mut remove_id = Option::None;
                        let mut replace_ids: Option<(u32, u32)> = Option::None;

                        for def in &item.affix_pools {
                            ui.horizontal(|ui| {
                                let (mut affix_pool_id, affix_pool_name) = {
                                    let affix_pool = def.lock().unwrap();

                                    (affix_pool.id, affix_pool.name.clone())
                                };

                                egui::ComboBox::from_id_source(format!("ItemAffixPools{:?}", def))
                                    .selected_text(format!("{}", affix_pool_name))
                                    .show_ui(ui, |ui| {
                                        let selected_pool_id = &mut affix_pool_id;
                                        let mut affix_pool_definitions =
                                            app.affix_pool_db.lock().unwrap().definitions();
                                        affix_pool_definitions.retain(|def| {
                                            def.lock().unwrap().id != *selected_pool_id
                                        });

                                        for pool in affix_pool_definitions {
                                            let pool_def = pool.lock().unwrap();
                                            ui.selectable_value(
                                                selected_pool_id,
                                                pool_def.id,
                                                pool_def.name.as_str(),
                                            );
                                        }
                                    });

                                if ui.button("Remove").clicked() {
                                    remove_id = Some(item.affix_pools.len() as u32 - 1);
                                }

                                let old_id = def.lock().unwrap().id;
                                if old_id != affix_pool_id {
                                    replace_ids = Some((old_id, affix_pool_id));
                                }
                            });
                        }

                        if let Some(id) = remove_id {
                            item.affix_pools
                                .retain(|pool| pool.lock().unwrap().id != id);
                        }

                        if let Some((old_id, new_id)) = replace_ids {
                            let item_idx = item
                                .affix_pools
                                .binary_search_by(|probe| probe.lock().unwrap().id.cmp(&old_id))
                                .unwrap();
                            item.affix_pools.remove(item_idx);
                            let new_def = app
                                .affix_pool_db
                                .lock()
                                .unwrap()
                                .definition(new_id)
                                .unwrap();
                            item.affix_pools.push(new_def);
                        }
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
                    ui.heading("Classification");
                });
                header.col(|ui| {
                    ui.heading("Pools");
                });
            })
            .body(|body| {
                let mut items = {
                    let item_db = app.item_db.lock().unwrap();
                    item_db.definitions()
                };

                // ZJ-TODO: other sorting/filtering methods? would be nice to sort/filter other columns, not just ID
                items.sort_by(|a, b| a.lock().unwrap().id.cmp(&b.lock().unwrap().id));

                body.rows(30., items.len(), |mut row| {
                    let item = items[row.index()].lock().unwrap();

                    if let Some(selected_definition_id) = app.selected_definition_id {
                        row.set_selected(selected_definition_id == item.id as u64);
                    }

                    app.populate_row_for_definition(&*item, &mut row);
                });
            });
    });
}
