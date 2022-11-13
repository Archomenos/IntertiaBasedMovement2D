use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::HashMap};
use noise::{NoiseFn, SuperSimplex};
use std::{collections::hash_map, time::Duration};
#[derive(Component)]
struct MovementGrid {
    grid: Vec<Vec<u8>>,
}
#[derive(Component)]
struct MoveCommand {
    target: Vec2,
}
#[derive(Component)]
struct Movable {}
#[derive(Resource)]
struct GridSettings {
    cell_size: f32,
    grid_width: u32,
    grid_height: u32,
    x_y_offset: Vec2,
    density: f64,
}
struct PathNode {
    pos: Vec2,
    heading: f64,
}
struct Path {
    path: Vec<PathNode>,
}
#[derive(Resource)]
struct AStarTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GridSettings {
            cell_size: 40.0,
            grid_width: 26,
            grid_height: 26,
            x_y_offset: Vec2::new(500.0, 500.0),
            density: 0.4,
        })
        .insert_resource(AStarTimer(Timer::new(
            Duration::from_millis(500),
            TimerMode::Repeating,
        )))
        .add_startup_system(setup)
        .add_startup_system(generate_grid)
        .add_startup_system_to_stage(
            bevy::app::StartupStage::PostStartup,
            generate_obstacles.after(generate_grid),
        )
        .add_system(calculate_a_star)
        // .add_system(print_grid)
        .run();
}

fn print_grid(mut movement_grid_q: Query<&mut MovementGrid>) {
    println!("___________________");
    match movement_grid_q.get_single_mut() {
        Ok(mut movement_grid) => {
            for j in 0..movement_grid.grid[0].len() as usize {
                for i in 0..movement_grid.grid.len() {
                    print!(
                        "|{}",
                        movement_grid.grid[i][movement_grid.grid[0].len() - 1 - j]
                    );
                }
                println!("|")
            }
        }
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    grid_settings: Res<GridSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    // Rectangle
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::DARK_GREEN,
            custom_size: Some(Vec2::new(1040.0, 1040.0)),
            ..default()
        },
        ..default()
    });
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("kill_me.png"),

            transform: Transform::from_scale(Vec3::new(0.05, 0.05, 0.05)).with_translation(
                Vec3::new(
                    // transform: Transform::from_scale(Vec3::new(0.5, 0.5, 0.5)).with_translation(Vec3::new(
                    0 as f32 * grid_settings.cell_size - grid_settings.x_y_offset.x,
                    0 as f32 * grid_settings.cell_size - grid_settings.x_y_offset.y,
                    0.0,
                ),
            ),
            ..default()
        })
        .insert(Movable {})
        .insert(MoveCommand {target : Vec2{x: 20.0, y: 20.0}});
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes
            .add(
                shape::Box::new(
                    grid_settings.cell_size,
                    grid_settings.cell_size,
                    grid_settings.cell_size,
                )
                .into(),
            )
            .into(),
        material: materials.add(ColorMaterial::from(Color::GOLD)),
        transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)).with_translation(Vec3::new(
            20 as f32 * grid_settings.cell_size - grid_settings.x_y_offset.x,
            20 as f32 * grid_settings.cell_size - grid_settings.x_y_offset.y,
            1.0,
        )),
        ..default()
    });
}

fn generate_grid(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    grid_settings: Res<GridSettings>,
) {
    let mut movement_grid: MovementGrid = MovementGrid { grid: Vec::new() };
    //    commands.spawn().insert(MovementGrid{
    //        grid: Vec::new()
    //    });
    for i in 0..grid_settings.grid_width as usize {
        movement_grid.grid.push(Vec::new());
        for j in 0..grid_settings.grid_height as usize {
            movement_grid.grid[i].push(0);
            commands.spawn_bundle(SpriteBundle {
                texture: asset_server.load("bloody_rectangle.png"),

                transform: Transform::from_scale(Vec3::new(0.5, 0.5, 0.5)).with_translation(
                    Vec3::new(
                        i as f32 * grid_settings.cell_size - grid_settings.x_y_offset.x,
                        j as f32 * grid_settings.cell_size - grid_settings.x_y_offset.y,
                        0.0,
                    ),
                ),
                ..default()
            });
        }
    }
    commands.spawn((movement_grid));
    println!("inserted");
}

fn generate_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid_settings: Res<GridSettings>,
    mut movement_grid_q: Query<&mut MovementGrid>,
) {
    let noise_generator: SuperSimplex = SuperSimplex::new(SuperSimplex::DEFAULT_SEED);
    //    let mut movement_grid : &MovementGrid;
    match movement_grid_q.get_single_mut() {
        Ok(mut movement_grid) => {
            for i in 0..grid_settings.grid_width as usize {
                for j in 0..grid_settings.grid_height as usize {
                    println!("{}", noise_generator.get([i as f64, j as f64]));
                    if noise_generator.get([i as f64, j as f64]) > grid_settings.density {
                        movement_grid.grid[i][j] = 1;
                        commands.spawn_bundle(MaterialMesh2dBundle {
                            mesh: meshes
                                .add(
                                    shape::Box::new(
                                        grid_settings.cell_size,
                                        grid_settings.cell_size,
                                        grid_settings.cell_size,
                                    )
                                    .into(),
                                )
                                .into(),
                            material: materials.add(ColorMaterial::from(Color::RED)),
                            transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0))
                                .with_translation(Vec3::new(
                                    i as f32 * grid_settings.cell_size - grid_settings.x_y_offset.x,
                                    j as f32 * grid_settings.cell_size - grid_settings.x_y_offset.y,
                                    1.0,
                                )),
                            ..default()
                        });
                    }
                }
            }
        }
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    }
}

fn move_unit(
    time: Res<Time>,
    mut timer: ResMut<AStarTimer>,
    mut movables: Query<(Entity, &mut Transform, &MoveCommand)>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (entity, transform, movecommand) in movables.iter() {}
    }
}
fn calculate_a_star(
    mut movables: Query<(Entity, &mut Transform, &MoveCommand)>,
    mut movement_grid_q: Query<&mut MovementGrid>,
    mut commands: Commands,
) {
    for (entity, transform, movcmd) in movables.iter() {
        if transform.translation.x == movcmd.target.x && transform.translation.y == movcmd.target.y
        {
            commands.entity(entity).remove::<MoveCommand>();
            continue;
        }
        match movement_grid_q.get_single_mut() {
            Ok(mut movement_grid) => {
                let mut last_step: Vec2 = Vec2 {
                    x: transform.translation.x,
                    y: transform.translation.y,
                };
                let mut open_set: HashMap<UVec2, u64> = HashMap::from([(
                    UVec2 {
                        x: transform.translation.x.floor() as u32,
                        y: transform.translation.y.floor() as u32,
                    },
                    0,
                )]);

                while !open_set.is_empty() {
                    // let mut current_node, current_cost : 
                    let mut count_vec: Vec<_> = open_set.iter().collect();
                    count_vec.sort_by_key(|a| a.1);
                    println!("lowest node {} with {}", count_vec[0].0, count_vec[0].1);
                }
            }
            Err(error) => {
                println!("{:?}", error);
                return;
            }
        }
    }
}
