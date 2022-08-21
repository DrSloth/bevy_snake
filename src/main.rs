//! Snake game implementation with bevy

use std::mem;
use bevy::{app::AppExit, ecs::system::EntityCommands, prelude::*, time::FixedTimestep};

use rand::{rngs::SmallRng, Rng, SeedableRng};

/// Field width from center to the right. Full width is this doubled
const FIELD_WIDTH: i16 = 10;
/// Field height from center to top. Full height is this doubled
const FIELD_HEIGHT: i16 = 10;
/// Size of the snake and the fruit
const SNAKE_SIZE: f32 = 50.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SmallRng::from_entropy())
        .add_startup_system(setup_system)
        .add_system(snake_input_system)
        .add_system(fruit_collision_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 5.0))
                .with_system(move_snake_system),
        )
        .run();
}

/// Setup the game
pub fn setup_system(mut commands: Commands, mut rng: ResMut<SmallRng>) {
    commands.spawn_bundle(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::Custom(Color::GRAY),
        },
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..Default::default()
    });

    // Spawn player
    create_snake_part(&mut commands, Vec3::ZERO).insert(SnakeHead {
        size: Vec2::splat(SNAKE_SIZE),
        ..Default::default()
    });

    let fruit_pos = gen_fruit_pos(&mut *rng);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::splat(SNAKE_SIZE)),
                ..Default::default()
            },
            transform: Transform::from_translation(fruit_pos),
            ..Default::default()
        })
        .insert(Fruit);

    let sprite = SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            // custom_size: Some(Vec2::splat(SNAKE_SIZE * f32::from(FIELD_WIDTH * 2))),
            ..Default::default()
        },
        transform: Transform::from_scale(Vec3::new(
            SNAKE_SIZE * (f32::from(FIELD_WIDTH) + 0.5) * 2.0,
            SNAKE_SIZE * (f32::from(FIELD_HEIGHT) + 0.5) * 2.0,
            -2.0,
        )),
        ..Default::default()
    };
    // commands.spawn_bundle(ImageBundle {
    //     image: UiImage(sprite.texture),
    //     style: Style {

    //         ..Default::default(),
    //     },
    //     ..Default::default()
    // });
    commands.spawn_bundle(sprite);
}

/// Create a part of the snake
pub fn create_snake_part<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    pos: Vec3,
) -> EntityCommands<'w, 's, 'a> {
    let mut ent = commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::new(SNAKE_SIZE, SNAKE_SIZE)),
            ..Default::default()
        },
        transform: Transform::from_translation(pos),
        ..Default::default()
    });
    ent.insert(SnakePart);
    ent
}

/// Move the snake with the last given direction
pub fn move_snake_system(
    mut snake_heads: Query<(&mut Transform, &SnakeHead)>,
    mut snake_parts: Query<&mut Transform, Without<SnakeHead>>,
    mut exit_event: EventWriter<AppExit>,
) {
    for (mut transform, snake_head) in snake_heads.iter_mut() {
        let mut prev = transform.translation;
        transform.translation += Vec3::new(
            snake_head.direction.x * snake_head.size.x,
            snake_head.direction.y * snake_head.size.y,
            0.0,
        );

        let pos = transform.translation;
        for part in snake_head.tail.iter().copied() {
            if let Ok(mut part) = snake_parts.get_mut(part) {
                if part.translation == pos {
                    exit_event.send(AppExit);
                }
                
                mem::swap(&mut part.translation, &mut prev);
            }
        }

        let x_bounds = (f32::from(FIELD_WIDTH.saturating_neg()) * SNAKE_SIZE)
            ..=(f32::from(FIELD_WIDTH) * SNAKE_SIZE);
        let y_bounds = (f32::from(FIELD_HEIGHT.saturating_neg()) * SNAKE_SIZE)
            ..=(f32::from(FIELD_HEIGHT) * SNAKE_SIZE);

        if !x_bounds.contains(&pos.x) || !y_bounds.contains(&pos.y) {
            exit_event.send(AppExit);
        }
    }
}

/// Get the keyborad input
pub fn snake_input_system(mut query: Query<&mut SnakeHead>, input: Res<Input<KeyCode>>) {
    for mut snake_head in query.iter_mut() {
        for key in input.get_just_pressed() {
            snake_head.direction = match key {
                KeyCode::A | KeyCode::Left => Vec2::new(-1.0, 0.0),
                KeyCode::D | KeyCode::Right => Vec2::new(1.0, 0.0),
                KeyCode::W | KeyCode::Up => Vec2::new(0.0, 1.0),
                KeyCode::S | KeyCode::Down => Vec2::new(0.0, -1.0),
                _ => continue,
            };
        }
    }
}

/// System that handles fruit collection
pub fn fruit_collision_system(
    mut fruits: Query<&mut Transform, With<Fruit>>,
    mut snake_heads: Query<(&Transform, &mut SnakeHead, Entity), Without<Fruit>>,
    snake_parts: Query<&Transform, (With<SnakePart>, Without<Fruit>)>,
    mut rng: ResMut<SmallRng>,
    mut commands: Commands,
) {
    for (snake_head_pos, mut snake_head, snake_head_entity) in snake_heads.iter_mut() {
        for mut fruit in fruits.iter_mut() {
            if snake_head_pos.translation == fruit.translation {
                fruit.translation = gen_fruit_pos(&mut *rng);

                let last_snake_part = snake_head.tail.last().copied().unwrap_or(snake_head_entity);
                if let Ok(last_snake_part) = snake_parts.get(last_snake_part) {
                    let new_snake_part = create_snake_part(
                        &mut commands,
                        last_snake_part.translation
                            - Vec3::new(
                                snake_head.direction.x * snake_head.size.x,
                                snake_head.direction.y * snake_head.size.y,
                                0.0,
                            ),
                    );
                    snake_head.tail.push(new_snake_part.id());
                }
            }
        }
    }
}

// Convert a Vec2 to a Vec3 by setting the z axis to 0
// pub fn vec2_to_vec3(v: Vec2) -> Vec3 {
//     Vec3::new(v.x, v.y, 0.0)
// }

/// Generate a fruit position inside the given bounds.
pub fn gen_fruit_pos<R: Rng>(rng: &mut R) -> Vec3 {
    let x: i16 = rng.gen_range(FIELD_WIDTH.saturating_neg()..=FIELD_WIDTH);
    let y: i16 = rng.gen_range(FIELD_HEIGHT.saturating_neg()..=FIELD_HEIGHT);

    Vec3::new(f32::from(x) * SNAKE_SIZE, f32::from(y) * SNAKE_SIZE, 0.0)
}

/// The snakes head
#[derive(Component, Debug, Default)]
pub struct SnakeHead {
    direction: Vec2,
    size: Vec2,
    tail: Vec<Entity>,
}

/// Any part of the snake
#[derive(Component, Debug)]
pub struct SnakePart;

/// A fruit for the snake to collect
#[derive(Component, Debug)]
pub struct Fruit;
