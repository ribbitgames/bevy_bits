pub mod controls;
pub mod inputs;

use avian3d::prelude::*;
use bevy::color::palettes::css::ORANGE;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::gameplay::JumpedEvent;

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct PlayerSettings {
    pub initial_position: Vec3,
    pub initial_rotation: f32,
    pub jump_velocity: f32,
}

#[derive(Resource, Deref, DerefMut)]
struct SmokeMaterialHandle(Handle<StandardMaterial>);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<inputs::Action>::default())
            .register_type::<PlayerSettings>()
            .insert_resource(PlayerSettings {
                jump_velocity: 10.0,
                initial_position: Vec3::new(0.0, 1.0, 0.0),
                initial_rotation: -0.28,
            })
            .insert_resource(SmokeMaterialHandle(Handle::default()))
            .add_systems(Startup, setup)
            .add_systems(Update, (gunshot_lighting, smoke_control));
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Smoke;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_settings: Res<PlayerSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut smoke_material_handle: ResMut<SmokeMaterialHandle>,
) {
    let bird = asset_server.load(GltfAssetLabel::Scene(0).from_asset("objects/bird.glb#Scene0"));
    let parent = commands
        .spawn((
            Name::new("Player"),
            Player,
            RigidBody::Dynamic,
            DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),
            GravityScale(4.0),
            LockedAxes::new()
                .lock_translation_x()
                .lock_translation_z()
                .lock_translation_y(),
            LinearVelocity::ZERO,
            Collider::cuboid(0.25, 1.3, 0.3),
            SceneRoot(bird),
            Transform::from_translation(player_settings.initial_position),
            InputManagerBundle::<inputs::Action> {
                input_map: inputs::create_input_map(),
                ..default()
            },
        ))
        .id();

    let smoke_material = StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
        ..default()
    };

    **smoke_material_handle = materials.add(smoke_material);

    let smoke = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::default().mesh().uv(16, 8))),
            MeshMaterial3d(smoke_material_handle.clone()),
            Transform::from_xyz(0.05, -0.81, 0.0),
            NotShadowCaster,
            Smoke,
        ))
        .id();

    let light1 = commands
        .spawn((
            PointLight {
                intensity: 0.0,
                color: Color::Srgba(ORANGE),
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(0.4, 0.0, 0.0),
        ))
        .id();

    let light2 = commands
        .spawn((
            PointLight {
                intensity: 0.0,
                color: Color::Srgba(ORANGE),
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(0.05, -0.81, 0.0),
        ))
        .id();

    commands
        .entity(parent)
        .add_children(&[light1, light2, smoke]);
}

fn smoke_control(
    mut materials: ResMut<Assets<StandardMaterial>>,
    smoke_material_handle: Res<SmokeMaterialHandle>,
    mut jump_event: EventReader<JumpedEvent>,
    mut alpha: Local<f32>,
    mut scale: Local<f32>,
    time: Res<Time>,
    mut smoke_query: Query<&mut Transform, With<Smoke>>,
) {
    let gunshot_event = !jump_event.is_empty();
    for _ in jump_event.read() {} // Clear the queue

    if gunshot_event {
        *alpha = 0.8;
        *scale = 0.0;
    }

    if let Some(material) = materials.get_mut(&smoke_material_handle.clone()) {
        material.base_color = Color::srgba(1.0, 1.0, 1.0, *alpha);
    }

    for mut transform in &mut smoke_query {
        transform.scale = Vec3::splat(*scale);
    }

    if *alpha > 0.0 {
        *alpha -= 5.0 * time.delta_secs();
        *scale += 10.0 * time.delta_secs();
    }
}

fn gunshot_lighting(
    mut light_query: Query<&mut PointLight>,
    mut jump_event: EventReader<JumpedEvent>,
) {
    let gunshot_event = !jump_event.is_empty();
    for _ in jump_event.read() {} // Clear the queue

    for mut light in &mut light_query {
        light.intensity -= 50_000.0;

        if light.intensity < 0.0 {
            light.intensity = 0.0;
        }

        if gunshot_event {
            light.intensity = 1_000_000.0;
        }
    }
}
