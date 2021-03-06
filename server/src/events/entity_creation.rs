use crate::{sys, Server, StateExt};
use common::{
    character::CharacterId,
    comp::{
        self, beam, humanoid::DEFAULT_HUMANOID_EYE_HEIGHT, shockwave, Agent, Alignment, Body,
        Gravity, Item, ItemDrop, LightEmitter, Loadout, Ori, Pos, Projectile, Scale, Stats, Vel,
        WaypointArea,
    },
    outcome::Outcome,
    util::Dir,
};
use comp::group;
use specs::{Builder, Entity as EcsEntity, WorldExt};
use vek::{Rgb, Vec3};

pub fn handle_initialize_character(
    server: &mut Server,
    entity: EcsEntity,
    character_id: CharacterId,
) {
    server.state.initialize_character_data(entity, character_id);
}

pub fn handle_loaded_character_data(
    server: &mut Server,
    entity: EcsEntity,
    loaded_components: (comp::Body, comp::Stats, comp::Inventory, comp::Loadout),
) {
    server
        .state
        .update_character_data(entity, loaded_components);
    sys::subscription::initialize_region_subscription(server.state.ecs(), entity);
}

#[allow(clippy::too_many_arguments)] // TODO: Pending review in #587
pub fn handle_create_npc(
    server: &mut Server,
    pos: Pos,
    stats: Stats,
    loadout: Loadout,
    body: Body,
    agent: impl Into<Option<Agent>>,
    alignment: Alignment,
    scale: Scale,
    drop_item: Option<Item>,
) {
    let group = match alignment {
        Alignment::Wild => None,
        Alignment::Passive => None,
        Alignment::Enemy => Some(group::ENEMY),
        Alignment::Npc | Alignment::Tame => Some(group::NPC),
        // TODO: handle
        Alignment::Owned(_) => None,
    };

    let entity = server
        .state
        .create_npc(pos, stats, loadout, body)
        .with(scale)
        .with(alignment);

    let entity = if let Some(group) = group {
        entity.with(group)
    } else {
        entity
    };

    let entity = if let Some(agent) = agent.into() {
        entity.with(agent)
    } else {
        entity
    };

    let entity = if let Some(drop_item) = drop_item {
        entity.with(ItemDrop(drop_item))
    } else {
        entity
    };

    entity.build();
}

#[allow(clippy::too_many_arguments)]
pub fn handle_shoot(
    server: &mut Server,
    entity: EcsEntity,
    dir: Dir,
    body: Body,
    light: Option<LightEmitter>,
    projectile: Projectile,
    gravity: Option<Gravity>,
    speed: f32,
) {
    let state = server.state_mut();

    let mut pos = state
        .ecs()
        .read_storage::<Pos>()
        .get(entity)
        .expect("Failed to fetch entity")
        .0;

    let vel = *dir * speed;

    // Add an outcome
    state
        .ecs()
        .write_resource::<Vec<Outcome>>()
        .push(Outcome::ProjectileShot { pos, body, vel });

    let eye_height = match state.ecs().read_storage::<comp::Body>().get(entity) {
        Some(comp::Body::Humanoid(body)) => body.eye_height(),
        _ => DEFAULT_HUMANOID_EYE_HEIGHT,
    };

    pos.z += eye_height;

    let mut builder = state.create_projectile(Pos(pos), Vel(vel), body, projectile);
    if let Some(light) = light {
        builder = builder.with(light)
    }
    if let Some(gravity) = gravity {
        builder = builder.with(gravity)
    }

    builder.build();
}

pub fn handle_shockwave(
    server: &mut Server,
    properties: shockwave::Properties,
    pos: Pos,
    ori: Ori,
) {
    let state = server.state_mut();
    state.create_shockwave(properties, pos, ori).build();
}

pub fn handle_beam(server: &mut Server, properties: beam::Properties, pos: Pos, ori: Ori) {
    let state = server.state_mut();
    state.create_beam(properties, pos, ori).build();
}

pub fn handle_create_waypoint(server: &mut Server, pos: Vec3<f32>) {
    server
        .state
        .create_object(Pos(pos), comp::object::Body::CampfireLit)
        .with(LightEmitter {
            col: Rgb::new(1.0, 0.3, 0.1),
            strength: 5.0,
            flicker: 1.0,
            animated: true,
        })
        .with(WaypointArea::default())
        .with(comp::Mass(100000.0))
        .build();
}
