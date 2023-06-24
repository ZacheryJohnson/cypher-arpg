use bevy::{
    prelude::{
        AssetServer, BuildChildren, Camera2dBundle, Color, Commands, NodeBundle, Res, TextBundle,
    },
    text::TextStyle,
    ui::{AlignItems, Display, FlexDirection, FlexWrap, Overflow, Size, Style, UiRect, Val},
    utils::default,
};

use crate::components::{ui_item_text::UiItemText, ui_item_text_box::UiItemTextBox};

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn((
            UiItemTextBox,
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(10.0), Val::Percent(20.0)),
                    flex_wrap: FlexWrap::Wrap,
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.03,
                    display: Display::Flex,
                    overflow: Overflow::Hidden,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                background_color: Color::rgba(0.15, 0.15, 0.15, 0.0).into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                UiItemText,
                TextBundle::from_section(
                    "foobar",
                    TextStyle {
                        font: asset_server.load("fonts/Exo-Regular.ttf"),
                        font_size: 15.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                }),
            ));
        });
}
