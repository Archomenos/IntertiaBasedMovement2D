use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    utils::{HashMap, HashSet},
};
use noise::{NoiseFn, SuperSimplex};
use std::{collections::hash_map, time::Duration};

const DISTANCE_FACTOR: f32 = 100.0;

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
        .insert(MoveCommand {
            target: Vec2 { x: 20.0, y: 20.0 },
        });
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

fn reconstruct_path(came_from: &HashMap<UVec2, UVec2>, end: UVec2) -> Vec<UVec2> {
    let mut total_path: Vec<UVec2> = vec![end];

    let mut current: UVec2 = end;
    while came_from.contains_key(&current) {
        current = came_from[&current];
        total_path.push(current);
    }
    println!("{:?}", total_path);
    return total_path;
}

fn calculate_a_star(
    mut movables: Query<(Entity, &mut Transform, &MoveCommand)>,
    mut movement_grid_q: Query<&mut MovementGrid>,
    mut commands: Commands,
) //-> Option<Vec<UVec2>>
{
    for (entity, transform, movcmd) in movables.iter() {
        if transform.translation.x == movcmd.target.x && transform.translation.y == movcmd.target.y
        {
            commands.entity(entity).remove::<MoveCommand>();
            continue;
        }
        match movement_grid_q.get_single_mut() {
            Ok(mut movement_grid) => {
                let start: UVec2 = UVec2 {
                    x: transform.translation.x.floor() as u32,
                    y: transform.translation.y.floor() as u32,
                };

                let mut f_score: HashMap<UVec2, u32> = HashMap::from([(
                    start,
                    (heuristical_distance(start, movcmd.target.as_uvec2()) * DISTANCE_FACTOR)
                        as u32,
                )]);
                let mut g_score: HashMap<UVec2, u32> = HashMap::from([(start, 0)]);
                let mut came_from: HashMap<UVec2, UVec2> = HashMap::new();
                let mut open_set: HashSet<UVec2> = HashSet::from([start]);
                while !open_set.is_empty() {
                    // let mut current_node, current_cost :
                    let mut count_vec: Vec<_> = f_score.iter().collect();
                    count_vec.sort_by_key(|a| a.1);
                    let current: (UVec2, u32) = (count_vec[0].0.clone(), count_vec[0].1.clone());
                    if current.0 == movcmd.target.as_uvec2() {
                        reconstruct_path(&came_from, current.0);
                    }
                    open_set.retain(|&x| x != current.0);
                    for neighbour in get_neighbours(&current.0, &movement_grid) {
                        println!("{}", neighbour);
                        let tentative_g_score: u32 = g_score[&current.0]
                            + (neighbour.as_vec2().distance(movcmd.target) * DISTANCE_FACTOR)
                                as u32;
                        let mut new_path: bool = false;
                        match g_score.get_mut(&neighbour) {
                            Some(n_g_score) => {
                                if tentative_g_score < *n_g_score {
                                    *n_g_score = tentative_g_score;
                                    new_path = true;
                                }
                            }
                            None => {
                                g_score.insert(neighbour, tentative_g_score);

                                new_path = true;
                            }
                        }
                        if new_path {
                            f_score.insert(
                                neighbour,
                                (heuristical_distance(neighbour, movcmd.target.as_uvec2())
                                    * DISTANCE_FACTOR) as u32,
                            );
                            came_from.insert(neighbour, current.0);
                            open_set.insert(neighbour);
                        }
                    }
                }
            }
            Err(error) => {
                println!("{:?}", error);
            }
        }
    }
    return; // None;
}

fn heuristical_distance(from: UVec2, to: UVec2) -> f32 {
    return from.as_vec2().distance(to.as_vec2());
}

fn get_neighbours(current: &UVec2, movement_grid: &MovementGrid) -> Vec<UVec2> {
    let mut adjacent_cells: Vec<UVec2> = Vec::new();
    for x in -1..2 {
        for y in -1..2 {
            let adjacent_cell: IVec2 = IVec2 {
                x: current.x as i32 + x,
                y: current.y as i32 + y,
            };
            if adjacent_cell.x >= 0
                && (adjacent_cell.x as usize) < movement_grid.grid.len()
                && adjacent_cell.y >= 0
                && (adjacent_cell.y as usize) < movement_grid.grid[0].len()
                && movement_grid.grid[adjacent_cell.x as usize][adjacent_cell.y as usize] == 0
            {
                adjacent_cells.push(UVec2 {
                    x: adjacent_cell.x as u32,
                    y: adjacent_cell.y as u32,
                });
            }
        }
    }
    return adjacent_cells;
}
