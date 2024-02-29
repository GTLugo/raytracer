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
    sprite: Sprite {
      flip_y: true,
      ..Default::default()
    },
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
  mut _image_settings: ResMut<RenderSettings>,
) {
  query.iter_mut().for_each(|image| {
    let Some(image) = images.get_mut(image.id()) else {
      return;
    };

    let UVec2 { x: width, y: height } = image.size();
    let aspect_ratio = width as f32 / height as f32;

    let mut iterator = image.data.iter_mut();
    'outer: for y in 0..height {
      for x in 0..width {
        let mut coord = [x as f32 / width as f32, y as f32 / height as f32];
        coord = [coord[0] * 2.0 - 1.0, coord[1] * 2.0 - 1.0];
        coord = [coord[0] * aspect_ratio, coord[1]];
        let Some(next) = iterator.next_tuple::<(&mut u8, &mut u8, &mut u8, &mut u8)>() else {
          break 'outer;
        };

        let color = pixel(coord);

        *next.0 = color[0];
        *next.1 = color[1];
        *next.2 = color[2];
        *next.3 = color[3];
      }
    }
  });
}

fn pixel(frag_pos: [f32; 2]) -> [u8; 4] {
  let mut color = [0, 0, 0, 255];

  let ray_origin = Vec3::new(0.0, 0.0, 2.0);
  let ray_dir = Vec3::new(frag_pos[0], frag_pos[1], -1.0).normalize();
  let radius = 0.5;

  color[0] = (ray_dir.x * 255.0) as u8;
  color[1] = (ray_dir.y * 255.0) as u8;

  let a = ray_dir.dot(ray_dir);
  let b = 2.0 * ray_origin.dot(ray_dir);
  let c = ray_origin.dot(ray_origin) - radius * radius;

  let discriminant = b * b - 4.0 * a * c;

  if discriminant >= 0.0 {
    [255, 255, 255, 255]
  } else {
    [0, 0, 0, 255]
  }
}

fn ui(mut contexts: EguiContexts, mut _image_settings: ResMut<RenderSettings>) {
  egui::Window::new("Settings")
    .anchor(Align2::LEFT_BOTTOM, (10., -10.))
    .resizable(false)
    .show(contexts.ctx_mut(), |ui| ui.label("Placeholder"));
}
