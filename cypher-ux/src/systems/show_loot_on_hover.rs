use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::{
    prelude::{AssetServer, Camera, Color, GlobalTransform, Query, Res, Transform, Vec2, With},
    text::{Text, TextSection, TextStyle},
    ui::BackgroundColor,
    window::{PrimaryWindow, Window},
};
use cypher_core::affix::instance::AffixInstance;
use cypher_world::components::dropped_item::DroppedItem;

use crate::{
    components::{ui_item_text::UiItemText, ui_item_text_box::UiItemTextBox},
    resources::player_settings::PlayerSettings,
};

pub fn show_loot_on_hover(
    mut ui_elements: Query<&mut BackgroundColor, With<UiItemTextBox>>,
    mut ui_text: Query<&mut Text, With<UiItemText>>,
    mut camera_query: Query<(&Camera, &GlobalTransform)>,
    dropped_items: Query<(&DroppedItem, &Transform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    player_settings: Res<PlayerSettings>,
) {
    let mut color = ui_elements.get_single_mut().unwrap();
    let mut text = ui_text.get_single_mut().unwrap();
    color.0 = Color::rgba(0.15, 0.15, 0.15, 0.0);
    text.sections.clear();

    let Ok((camera, camera_transform)) = camera_query.get_single_mut() else {
        println!("failed to query camera");
        return;
    };

    let window = window_query
        .get_single()
        .expect("failed to get primary camera");

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        // Couldn't convert - mouse likely outside of window
        // Don't log - this would get spammy
        return;
    };

    for (item_drop, item_transform) in &dropped_items {
        let world_pos_collider = Aabb2d::new(world_pos, Vec2 { x: 10.0, y: 10.0 });
        let item_collider = Aabb2d::new(
            item_transform.translation.truncate(),
            Vec2 { x: 10.0, y: 10.0 },
        );
        if world_pos_collider.intersects(&item_collider) {
            let item_instance = item_drop.item_instance.clone();

            color.0 = Color::rgba(0.15, 0.15, 0.15, 1.0);
            text.sections.push(TextSection {
                value: item_instance
                    .lock()
                    .unwrap()
                    .definition
                    .lock()
                    .unwrap()
                    .name
                    .clone(),
                style: TextStyle {
                    font: asset_server.load("fonts/Exo-Regular.ttf"),
                    font_size: 15.0,
                    color: Color::WHITE,
                },
            });
            for affix in &item_instance.lock().unwrap().affixes {
                add_affix_to_display(
                    affix,
                    &mut text,
                    player_settings.alt_mode_enabled,
                    &asset_server,
                );
            }
            break;
        }
    }
}

fn add_affix_to_display(
    affix: &AffixInstance,
    text_component: &mut Text,
    should_display_tier: bool,
    asset_server: &AssetServer,
) {
    let mut affix_str = "\n".to_owned() + &affix.stats.to_string();
    if should_display_tier {
        affix_str += format!(" (T{})", affix.tier).as_str();
    }
    text_component.sections.push(TextSection {
        value: affix_str,
        style: TextStyle {
            font: asset_server.load("fonts/Exo-Regular.ttf"),
            font_size: 15.0,
            color: Color::GOLD,
        },
    })
}
