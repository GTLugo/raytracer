use bevy::{
  prelude::*,
  window::{PresentMode, WindowResolution},
};

use self::raytracer::RaytracerPlugin;

mod raytracer;

fn main() {
  App::new()
    .add_plugins((
      DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          title: "Raytracer".to_owned(),
          resolution: WindowResolution::new(800., 600.),
          visible: false,
          present_mode: PresentMode::AutoNoVsync,
          ..Default::default()
        }),
        ..Default::default()
      }),
      RaytracerPlugin,
    ))
    .run();
}
