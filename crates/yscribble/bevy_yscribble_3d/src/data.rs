//! Manages access to [ScribbleData] so that fine-grained reactivity
//! can be achieved for the bevy ecs [World].

use std::{collections::HashSet, ops::DerefMut};

use bevy::ecs::system::{EntityCommands, SystemParam};

use crate::prelude::*;

pub(crate) struct DataPlugin;

impl Plugin for DataPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<ScribbleDataComponent>();
	}
}

#[derive(Component, Reflect, Default, Debug, Deref, DerefMut)]
pub(crate) struct ScribbleDataComponent(yscribble::prelude::ScribbleData);

impl ScribbleDataComponent {
	fn downgrade<'w>(
		this: Mut<'w, ScribbleDataComponent>,
	) -> &'w mut yscribble::prelude::ScribbleData {
		this.into_inner()
	}
}

#[derive(SystemParam)]
pub struct ScribbleData<'w, 's> {
	pads: Query<
		'w,
		's,
		(
			Entity,
			&'static PadConfig,
			&'static mut ScribbleDataComponent,
			&'static Children,
		),
	>,
	complete_spawner: Query<'w, 's, Entity, With<CompleteLineSpawnerMarker>>,
	complete_commands: Commands<'w, 's>,
	partial_spawner: Query<'w, 's, Entity, With<PartialLineSpawnerMarker>>,
	partial_commands: Commands<'w, 's>,
	mma: MMA<'w>,
}

impl<'s> ScribbleData<'s, 's> {
	fn pad_entity_from_detector(&self, detector_entity: Entity) -> Option<Entity> {
		self
			.pads
			.iter()
			.filter_map(|(pad, _, _, children)| {
				children
					.iter()
					.find(|child| *child == &detector_entity)
					.map(|_child_and_detector_entity| pad)
			})
			.next()
	}

	/// Constructs [PadData] from the detector entity
	pub(crate) fn with_detector(&'s mut self, detector_entity: Entity) -> Option<PadData<'s>> {
		let pad_entity = self.pad_entity_from_detector(detector_entity)?;
		let (_pad_entity, config, data, children) = self.pads.get_mut(pad_entity).ok()?;
		let complete_spawners = children
			.iter()
			.filter_map(|child| self.complete_spawner.get(*child).ok())
			.collect::<Vec<_>>();
		if complete_spawners.len() > 1 {
			error!(
				internal_error = true,
				message = "Multiple [Entity]s with [CompleteLineSpawnerMarker] found",
				note = "As children of a [PadSpawner]",
				?complete_spawners
			);
		}
		let complete_spawner = self
			.complete_commands
			.entity(complete_spawners.into_iter().next()?);

		let partial_spawners = children
			.iter()
			.filter_map(|child| self.partial_spawner.get(*child).ok())
			.collect::<Vec<_>>();
		if partial_spawners.len() > 1 {
			error!(
				internal_error = true,
				message = "Multiple [Entity]s with [PartialLineSpawnerMarker] found",
				note = "As children of a [PadSpawner]"
			);
		}
		let partial_spawner = self
			.partial_commands
			.entity(partial_spawners.into_iter().next()?);

		Some(PadData {
			data: ScribbleDataComponent::downgrade(data),
			config,
			complete_spawner,
			partial_spawner,
		})
	}
}

pub struct PadData<'s> {
	data: &'s mut yscribble::prelude::ScribbleData,
	config: &'s PadConfig,
	complete_spawner: EntityCommands<'s>,
	partial_spawner: EntityCommands<'s>,
}

// impl PadData<'_> {
// 	pub fn cut_line(&mut self) {}
// }
