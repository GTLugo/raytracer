use bevy::{
  core::FrameCount,
  diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
  prelude::*,
  render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
  },
};
use bevy_egui::{EguiContexts, EguiPlugin};
use egui::Align2;
use itertools::Itertools;

#[derive(Resource)]
pub struct RenderSettings {}

#[derive(Component)]
pub struct RenderTexture;

pub struct RaytracerPlugin;

impl Plugin for RaytracerPlugin {
  fn build(&self, app: &mut App) {
    app.insert_resource(RenderSettings {});

    app.add_plugins((LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin, EguiPlugin));

    app.add_systems(Startup, start);

    app.add_systems(Update, (make_visible, (resize, render).chain(), ui));
  }
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
  if frames.0 == 10 {
    window.single_mut().visible = true;
  }
}

fn start(mut commands: Commands, asset_server: Res<AssetServer>, mut window: Query<&mut Window>) {
  let window = window.single_mut();

  commands.spawn(Camera2dBundle::default());
  commands.spawn((RenderTexture, SpriteBundle {
    texture: asset_server.add(Image::new_fill(
      Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
        depth_or_array_layers: 1,
      },
      TextureDimension::D2,
      &[255, 255, 255, 255],
      TextureFormat::Rgba8UnormSrgb,
      RenderAssetUsages::all(),
    )),
    ..Default::default()
  }));
}

fn resize(
  mut query: Query<&mut Handle<Image>, With<RenderTexture>>,
  mut window: Query<&mut Window>,
  mut images: ResMut<Assets<Image>>,
) {
  let window = window.single_mut();

  query.iter_mut().for_each(|image| {
    let Some(image) = images.get_mut(image.id()) else {
      return;
    };

    // resize texture
    if image.width() != window.physical_width() || image.height() != window.physical_height() {
      // early resize data so that I might fill it with 255 so alpha is fully opaque
      image
        .data
        .resize((4 * window.physical_width() * window.physical_height()) as usize, 255);
      image.resize(Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
        depth_or_array_layers: 1,
      });
    }
  });
}

fn render(
  mut query: Query<&mut Handle<Image>, With<RenderTexture>>,
  mut images: ResMut<Assets<Image>>,
  mut image_settings: ResMut<RenderSettings>,
) {
  query.iter_mut().for_each(|image| {
    let Some(image) = images.get_mut(image.id()) else {
      return;
    };

    let width = image.width();
    let height = image.height();
    let mut i = 0;
    let mut iter = image.data.iter_mut();
    while let Some(pixel_tuple) = iter.next_tuple::<(&mut u8, &mut u8, &mut u8, &mut u8)>() {
      let x = i as f32 / width as f32;
      let y = i as f32 % height as f32;
      
      let color = pixel([x, y]);

      *pixel_tuple.0 = color[0];
      *pixel_tuple.1 = color[1];
      *pixel_tuple.2 = color[2];
      *pixel_tuple.3 = color[3];

      i += 1;
    }
  });
}

fn pixel(frag_pos: [f32; 2]) -> [u8; 4] {
  let mut color = [255; 4];

  color[0] = fastrand::u8(0..=255);
  color[1] = fastrand::u8(0..=255);
  color[2] = fastrand::u8(0..=255);

  color
}

fn ui(mut contexts: EguiContexts, mut image_settings: ResMut<RenderSettings>) {
  egui::Window::new("Settings")
    .anchor(Align2::LEFT_BOTTOM, (10., -10.))
    .resizable(false)
    .show(contexts.ctx_mut(), |ui| ui.label("Placeholder"));
}
