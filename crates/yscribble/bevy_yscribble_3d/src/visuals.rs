//! Internally uses the 'up' plane direction of -z, and right plane direction of +x

use crate::prelude::*;

pub struct YScribble3DVisuals;

impl Plugin for YScribble3DVisuals {
	fn build(&self, app: &mut App) {
		app
			.add_systems(Update, expand_pad_bundles)
			.register_type::<PadConfig>();
	}
}

pub use config::*;
mod config {
	use crate::prelude::*;

	/// Rectangular scribble pad.
	#[derive(Bundle, Debug)]
	pub struct PadBundle {
		pub config: PadConfig,

		pub visibility: VisibilityBundle,
		pub transform: TransformBundle,
		pub name: Name,
	}

	#[derive(Component, Reflect, Debug)]
	pub struct PadConfig {
		pub width: f32,
		pub height: f32,

		pub depth: f32,
	}

	impl Default for PadConfig {
		fn default() -> Self {
			PadConfig {
				width: 10.0,
				height: 10.0,

				depth: 0.1,
			}
		}
	}

	impl Default for PadBundle {
		fn default() -> Self {
			PadBundle {
				name: Name::new("Scribble Pad (Parent)"),
				config: PadConfig::default(),
				transform: Default::default(),
				visibility: Default::default(),
			}
		}
	}
}

fn expand_pad_bundles(
	bundles: Query<(Entity, &PadConfig), (Added<PadConfig>, Without<Children>)>,
	mut commands: Commands,
	mut meshs: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	ass: Res<AssetServer>,
) {
	for (entity, config) in bundles.iter() {
		// debug!(
		// 	message = "Expanding a pad bundle into a scribble pad",
		// 	?config
		// );
		let PadConfig {
			width,
			height,
			depth,
		} = config;
		let half_width = *width / 2.0;
		let half_height = *height / 2.0;
		let just_above_depth = depth * 1.2;

		commands.entity(entity).with_children(|parent| {
			parent.spawn((
				PbrBundle {
					mesh: meshs.add(Cuboid::new(*width, *depth, *height)),
					material: materials.add(Color::GRAY),
					..default()
				},
				On::<Pointer<DragStart>>::run(on_drag_start),
				PickableBundle::default(),
				Name::new("Pickable surface"),
			));

			parent.spawn((
				PbrBundle {
					mesh: meshs.add(Cuboid::new(width * 0.95, *depth, *depth)),
					material: materials.add(Color::WHITE),
					transform: Transform::from_translation(Vec3::new(0.0, *depth + 0.1, -half_height)),
					..default()
				},
				Name::new("Pad Outline Top"),
			));

			parent.spawn((
				SceneBundle {
					// arrow points in -z direction
					transform: Transform::from_scale(Vec3::splat(0.05)).with_translation(Vec3::new(
						half_width * 0.95,
						just_above_depth,
						-half_height * 0.9,
					)),
					scene: ass.load("blender/Arrow.glb#Scene0"),
					..default()
				},
				Name::new("Arrow Model"),
			));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn non_recursive_default() {
		let _config: PadConfig = PadConfig::default();
	}
}