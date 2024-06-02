use crate::table_display::TableDisplay;
use crate::DataEditorApp;
use cypher_core::data::DataDefinitionDatabase;
use cypher_item::item::classification::ItemClassification;
use cypher_item::item::definition::ItemDefinition;
use cypher_item::loot_pool::definition::LootPoolDefinition;
use cypher_item::loot_pool::member::LootPoolMember;
use egui_extras::{Column, TableBuilder};
use std::sync::{Arc, Mutex};

impl TableDisplay for LootPoolDefinition {
    fn header_row_values() -> Vec<&'static str> {
        vec!["Id", "Name", "Members"]
    }

    fn data_row_values(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.members.len().to_string(),
        ]
    }
}

pub fn draw(ctx: &egui::Context, app: &mut DataEditorApp) {
    egui::TopBottomPanel::top("loot_pool_menu_bar").show(ctx, |ui| {
        if ui.button("Add").clicked() {
            app.invalidate_selections();

            let mut loot_pool_db = app.loot_pool_db.lock().unwrap();
            let loot_pools = loot_pool_db.definitions();
            let next_id = loot_pools.iter().fold(0, |acc, next| {
                if acc > next.lock().unwrap().id {
                    acc
                } else {
                    next.lock().unwrap().id
                }
            }) + 1;
            let new_loot_pool = LootPoolDefinition {
                id: next_id,
                name: String::new(),
                members: vec![],
            };

            loot_pool_db.add_definition(new_loot_pool);
        }
    });

    if app.selected_definition_id.is_some() {
        egui::SidePanel::right("loot_pool_right_panel")
            .min_width(300.)
            .show(ctx, |ui| {
                if ui.button("Close").clicked() {
                    app.invalidate_selections();
                    return;
                }
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let loot_pool_db = app.loot_pool_db.lock().unwrap();
                    let mut loot_pools = loot_pool_db.definitions();
                    let mut loot_pool = loot_pools
                        .iter_mut()
                        .find(|def| {
                            def.lock().unwrap().id as u64 == app.selected_definition_id.unwrap()
                        })
                        .unwrap()
                        .lock()
                        .unwrap();

                    ui.label(format!("Id: {}", loot_pool.id));
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut loot_pool.name);
                    });

                    ui.separator();

                    for member in &mut loot_pool.members {
                        let member_def = member.item_def.lock().unwrap();
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", member_def.name));
                            ui.horizontal(|ui| {
                                ui.label("Weight");
                                ui.add(egui::DragValue::new(&mut member.weight));
                            });
                        });
                    }

                    ui.separator();

                    {
                        ui.horizontal(|ui| {
                            ui.label("Members");
                            if ui.button("Add").clicked() {
                                let next_id = loot_pool.members.len() as u32;
                                loot_pool.members.insert(
                                    next_id as usize,
                                    LootPoolMember {
                                        item_def: Arc::new(Mutex::new(ItemDefinition {
                                            id: 0,
                                            classification: ItemClassification::Invalid,
                                            affix_pools: vec![],
                                            fixed_affixes: vec![],
                                            name: "".to_string(),
                                        })),
                                        weight: 0,
                                    },
                                );
                            }
                        });

                        let mut remove_id: Option<u64> = None;
                        let mut replace_ids: Option<(u64, u64)> = None;

                        for member in &loot_pool.members {
                            let (mut member_id, member_name) = {
                                let member_mutex_guard = member.item_def.lock().unwrap();
                                (member_mutex_guard.id, member_mutex_guard.name.clone())
                            };

                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_source(format!(
                                    "LootPoolMember{:?}",
                                    member_id
                                ))
                                .selected_text(format!("{}", member_name))
                                .show_ui(ui, |ui| {
                                    let selected_member_id = &mut member_id;
                                    let mut item_defs = app.item_db.lock().unwrap().definitions();
                                    item_defs.retain(|def| {
                                        let item_id = def.lock().unwrap().id;
                                        let is_member_already =
                                            loot_pool.members.iter().any(|member| {
                                                member.item_def.lock().unwrap().id == item_id
                                            });
                                        let is_selected_item =
                                            item_id as u64 == *selected_member_id as u64;

                                        !(is_member_already || is_selected_item)
                                    });

                                    for item in item_defs {
                                        let item_def = item.lock().unwrap();
                                        ui.selectable_value(
                                            selected_member_id,
                                            item_def.id,
                                            item_def.name.as_str(),
                                        );
                                    }
                                });

                                if ui.button("Remove").clicked() {
                                    remove_id = Some(loot_pool.members.len() as u64 - 1);
                                }

                                let old_id = member.item_def.lock().unwrap().id;
                                if old_id != member_id {
                                    replace_ids = Some((old_id, member_id));
                                }
                            });
                        }

                        if let Some(id) = remove_id {
                            loot_pool
                                .members
                                .retain(|member| member.item_def.lock().unwrap().id != id);
                        }

                        if let Some((old_id, new_id)) = replace_ids {
                            let item_idx = loot_pool
                                .members
                                .binary_search_by(|probe| {
                                    probe.item_def.lock().unwrap().id.cmp(&old_id)
                                })
                                .unwrap();
                            loot_pool.members.remove(item_idx);
                            let new_def = app.item_db.lock().unwrap().definition(new_id).unwrap();
                            loot_pool.members.push(LootPoolMember {
                                item_def: new_def,
                                weight: 1,
                            });
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
                for header_text in LootPoolDefinition::header_row_values() {
                    header.col(|ui| {
                        ui.heading(header_text);
                    });
                }
            })
            .body(|body| {
                let mut loot_pools = {
                    let loot_pool_db = app.loot_pool_db.lock().unwrap();
                    loot_pool_db.definitions()
                };

                // ZJ-TODO: other sorting/filtering methods? would be nice to sort/filter other columns, not just ID
                loot_pools.sort_by(|a, b| a.lock().unwrap().id.cmp(&b.lock().unwrap().id));

                body.rows(30., loot_pools.len(), |mut row| {
                    let loot_pool = loot_pools[row.index()].lock().unwrap();

                    if let Some(selected_definition_id) = app.selected_definition_id {
                        row.set_selected(selected_definition_id == loot_pool.id as u64);
                    }

                    app.populate_row_for_definition(&*loot_pool, &mut row);
                });
            });
    });
}
