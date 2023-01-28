#![feature(variant_count)]
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use noise::{NoiseFn, SuperSimplex};
use std::{
    collections::{hash_map, HashMap, HashSet},
    mem,
    time::Duration,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const DISTANCE_FACTOR: f32 = 100.0;
#[derive(Eq, PartialEq, Hash, Clone, Copy, EnumIter, Debug)]
enum Heading {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

#[derive(Component)]
struct MovementGrid {
    grid: Vec<Vec<u8>>,
}
#[derive(Component)]
struct MoveCommand {
    target: Vec2,
    path: Vec<(UVec2, Heading)>,
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
fn calculate_base_inertia(heading_in: Heading, heading_out: Heading) -> u32 {
    println!("Heading in {:?}, Heading out {:?}", heading_in, heading_out);
    let mut penalty: u32 = 0;
    let difference: i32 = (heading_out as i32 - heading_in as i32).abs();
    let half_headings: i32 = (Heading::iter().len() as f32 / 2.0).ceil() as i32;
    println!("difference {} half_headings {}", difference, half_headings);
    // if difference.abs() > half_headings {
    penalty = (half_headings - (difference - half_headings).abs()) as u32;
    // }
    println!("penalty {}", penalty);
    return penalty;
}
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
            Duration::from_millis(1500),
            TimerMode::Repeating,
        )))
        .add_startup_system(setup)
        .add_startup_system(generate_grid)
        .add_startup_system_to_stage(
            bevy::app::StartupStage::PostStartup,
            generate_obstacles.after(generate_grid),
        )
        .add_system(calculate_a_star)
        .add_system(visualise_path)
        .add_system(print_grid)
        .run();
}

fn print_grid(mut gridmap_q: Query<&mut MovementGrid>) {
    println!("___________________");
    match gridmap_q.get_single_mut() {
        Ok(mut gridmap) => {
            for j in 0..gridmap.grid[0].len() as usize {
                for i in 0..gridmap.grid.len() {
                    print!("|{}", gridmap.grid[i][gridmap.grid[0].len() - 1 - j]);
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
    let target: Vec2 = Vec2 { x: 7.0, y: 20.0 };
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
        .insert(MoveCommand {
            target: target,
            path: Vec::new(),
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
            target.x * grid_settings.cell_size - grid_settings.x_y_offset.x,
            target.y as f32 * grid_settings.cell_size - grid_settings.x_y_offset.y,
            1.0,
        )),
        ..default()
    });
}
// fn add_square(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     grid_settings: Res<GridSettings>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     pos: UVec2,
//     color: ColorMaterial,
// ) -> MaterialMeshBundle {
//     *commands.spawn_bundle(MaterialMesh2dBundle {
//         mesh: meshes
//             .add(
//                 shape::Box::new(
//                     grid_settings.cell_size,
//                     grid_settings.cell_size,
//                     grid_settings.cell_size,
//                 )
//                 .into(),
//             )
//             .into(),
//         material: materials.add(color),
//         transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)).with_translation(Vec3::new(
//             pos.x as f32 * grid_settings.cell_size - grid_settings.x_y_offset.x,
//             pos.y as f32 * grid_settings.cell_size - grid_settings.x_y_offset.y,
//             1.0,
//         )),
//         ..default()
//     });
// }
fn generate_grid(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    grid_settings: Res<GridSettings>,
) {
    let mut gridmap: MovementGrid = MovementGrid { grid: Vec::new() };
    //    commands.spawn().insert(MovementGrid{
    //        grid: Vec::new()
    //    });
    for i in 0..grid_settings.grid_width as usize {
        gridmap.grid.push(Vec::new());
        for j in 0..grid_settings.grid_height as usize {
            gridmap.grid[i].push(0);
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
    commands.spawn((gridmap));
    println!("inserted");
}

fn generate_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    grid_settings: Res<GridSettings>,
    mut gridmap_q: Query<&mut MovementGrid>,
) {
    let noise_generator: SuperSimplex = SuperSimplex::new(SuperSimplex::DEFAULT_SEED);
    //    let mut gridmap : &MovementGrid;
    match gridmap_q.get_single_mut() {
        Ok(mut gridmap) => {
            for i in 0..grid_settings.grid_width as usize {
                for j in 0..grid_settings.grid_height as usize {
                    // println!("{}", noise_generator.get([i as f64, j as f64]));
                    if noise_generator.get([i as f64, j as f64]) > grid_settings.density {
                        gridmap.grid[i][j] = 1;
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

fn reconstruct_path(
    came_from: &HashMap<(UVec2, Heading), (UVec2, Heading)>,
    end: (UVec2, Heading),
) -> Vec<(UVec2, Heading)> {
    let mut total_path: Vec<(UVec2, Heading)> = vec![end.clone()];

    let mut current: (UVec2, Heading) = end;
    while came_from.contains_key(&current) {
        current = came_from[&current];
        total_path.push(current.clone());
        println!("{:?}", current);
    }
    println!("{:?}", total_path);
    return total_path;
}
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct AStarNode {
    f_score: i32,
    g_score: i32,
    came_from: Option<UVec2>,
}
// impl PartialEq for AStarNode {
//     fn eq(&self, other: &Self) -> bool {
//         return self.pos == other.pos && self.heading == other.heading;
//     }
// }
fn calculate_a_star(
    mut movables: Query<(Entity, &mut Transform, &mut MoveCommand), Without<Movable>>,
    mut gridmap_q: Query<&mut MovementGrid>,
    mut commands: Commands,
) //-> Option<Vec<UVec2>>
{
    for (entity, transform, mut movcmd) in movables.iter_mut() {
        if transform.translation.x == movcmd.target.x && transform.translation.y == movcmd.target.y
        {
            commands.entity(entity).remove::<MoveCommand>();
            continue;
        }
        match gridmap_q.get_single_mut() {
            Ok(gridmap) => {
                let target: UVec2 = movcmd.target.as_uvec2();

                let mut movement_grid: Vec<Vec<HashMap<Heading, AStarNode>>> = vec![
                        vec![Heading::iter()
                            .map(|x| (
                                x.clone(),
                                AStarNode {
                                    f_score: -1,
                                    g_score: -1,
                                    came_from: None
                                }
                            ))
                            .into_iter()
                            .collect();
                        gridmap.grid.len()];
                gridmap.grid[0].len()
                    ];
                // println!("X_Length: {}, Y_Length: {}, Headings: {}", gridmap.grid.len(), gridmap)
                let mut came_from: HashMap<(UVec2, Heading), (UVec2, Heading)> = HashMap::new();
                let mut open_set: HashSet<(UVec2, Heading)> = HashSet::from([(
                    UVec2 {
                        x: transform.translation.x.floor() as u32,
                        y: transform.translation.y.floor() as u32,
                    },
                    Heading::N,
                )]);
                movement_grid[transform.translation.x.floor() as usize]
                    [transform.translation.y.floor() as usize]
                    .get_mut(&Heading::N)
                    .unwrap()
                    .g_score = 0;
                while !open_set.is_empty() {
                    let mut current: (UVec2, Heading) = (UVec2::ZERO, Heading::N);

                    let mut current_cost = 0;
                    for open_cell in open_set.clone() {
                        let mut cell: &AStarNode = movement_grid[open_cell.0.x as usize]
                            [open_cell.0.y as usize]
                            .get_mut(&open_cell.1)
                            .unwrap();
                        let cell_f_score: i32 = cell.f_score;
                        if current_cost == 0 || cell_f_score < current_cost {
                            current = open_cell;
                            current_cost = cell_f_score;
                        }
                    }

                    let current_node: AStarNode = movement_grid[current.0.x as usize]
                        [current.0.y as usize]
                        .get(&current.1)
                        .unwrap()
                        .to_owned();
                    if current.0 == movcmd.target.as_uvec2() {
                        for node in reconstruct_path(&came_from, current) {
                            if !(node.0.x == transform.translation.x.floor() as u32
                                && node.0.y == transform.translation.y.floor() as u32)
                                && node.0 != movcmd.target.as_uvec2()
                            {
                                movcmd.path.push(node);
                                println!("{:?}", node);
                            }

                            commands.entity(entity).insert(Movable {});
                        }
                        return;
                    }
                    open_set.remove(&current);
                    let neighbours = get_neighbours(current.0, &gridmap);
                    // println!("Current: {:?}", current);
                    for neighbour in neighbours {
                        // println!("{:?}", neighbour);
                        let mut neighbour_node: &mut AStarNode = movement_grid
                            [neighbour.0.x as usize][neighbour.0.y as usize]
                            .get_mut(&neighbour.1)
                            .unwrap();
                        let tentative_g_score: i32 = current_node.g_score
                            + (inertia_based_inter_cell_movement(current, neighbour)
                                * DISTANCE_FACTOR) as i32;

                        if tentative_g_score < neighbour_node.g_score
                            || neighbour_node.g_score == -1
                        {
                            neighbour_node.g_score = tentative_g_score;
                            neighbour_node.f_score = tentative_g_score
                                + (heuristical_distance(neighbour, (target, None))
                                    * DISTANCE_FACTOR) as i32;
                            came_from.insert(neighbour, current);
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

fn visualise_path(
    mut movables: Query<(Entity, &mut Transform, &mut MoveCommand), With<Movable>>,
    mut gridmap_q: Query<&mut MovementGrid>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    grid_settings: Res<GridSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut timer: ResMut<AStarTimer>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        timer.0.set_duration(Duration::from_millis(150));
        for (entity, transform, mut movcmd) in movables.iter_mut() {
            let node: (UVec2, Heading);
            match movcmd.path.pop() {
                Some(n) => node = n,
                None => {
                    commands.entity(entity).remove::<MoveCommand>();
                    commands.entity(entity).remove::<Movable>();

                    continue;
                }
            }
            commands.spawn(MaterialMesh2dBundle {
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
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)).with_translation(
                    Vec3::new(
                        node.0.x as f32 * grid_settings.cell_size - grid_settings.x_y_offset.x,
                        node.0.y as f32 * grid_settings.cell_size - grid_settings.x_y_offset.y,
                        1.0,
                    ),
                ),
                ..default()
            });
        }
    }
}

// TBD
fn inertia_based_inter_cell_movement(from: (UVec2, Heading), to: (UVec2, Heading)) -> f32 {
    let inertia: f32 = 0.0;
    let penalty: f32 = calculate_base_inertia(from.1, to.1) as f32;

    let cost: f32 = from.0.as_vec2().distance(to.0.as_vec2()).abs() + penalty;
    // println!(
    //     "from {:?} to {:?} penalty {:?}, cost {:?}",
    //     from, to, penalty, cost
    // );
    return cost;
}
fn heuristical_distance(from: (UVec2, Heading), to: (UVec2, Option<Heading>)) -> f32 {
    return from.0.as_vec2().distance(to.0.as_vec2());
}
fn calculate_heading(from: &UVec2, to: &UVec2) -> Heading {
    let diff: IVec2 = from.as_ivec2() - to.as_ivec2();
    let heading: Heading;
    if diff.x == 1 && diff.y == 0 {
        heading = Heading::N
    } else if diff.x == 1 && diff.y == 1 {
        heading = Heading::NE
    } else if diff.x == 0 && diff.y == 1 {
        heading = Heading::E
    } else if diff.x == -1 && diff.y == 1 {
        heading = Heading::SE
    } else if diff.x == -1 && diff.y == 0 {
        heading = Heading::S
    } else if diff.x == -1 && diff.y == -1 {
        heading = Heading::SW
    } else if diff.x == 0 && diff.y == -1 {
        heading = Heading::W
    } else {
        heading = Heading::NW
    }
    return heading;
}
fn check_path_width(current: UVec2, target: UVec2, gridmap: &MovementGrid) -> bool {
    if current.x != target.x && current.y != target.y {
        if gridmap.grid[current.x as usize][target.y as usize] != 0
            && gridmap.grid[target.x as usize][current.y as usize] != 0
        {
            println!("current {} neighbour {}", current, target);
            return false;
        }
    }
    return true;
}
fn get_neighbours(current: UVec2, gridmap: &MovementGrid) -> Vec<(UVec2, Heading)> {
    let mut neighbours: Vec<(UVec2, Heading)> = Vec::new();
    for x in -1..2 {
        for y in -1..2 {
            let adjacent_cell: IVec2 = IVec2 {
                x: current.x as i32 + x,
                y: current.y as i32 + y,
            };

            if adjacent_cell.x >= 0
                && (adjacent_cell.x as usize) < gridmap.grid.len()
                && adjacent_cell.y >= 0
                && (adjacent_cell.y as usize) < gridmap.grid[0].len()
                && gridmap.grid[adjacent_cell.x as usize][adjacent_cell.y as usize] == 0
                && adjacent_cell.as_uvec2() != current
                && check_path_width(current, adjacent_cell.as_uvec2(), &gridmap)
            {
                neighbours.push((
                    UVec2 {
                        x: adjacent_cell.x as u32,
                        y: adjacent_cell.y as u32,
                    },
                    calculate_heading(&current, &adjacent_cell.as_uvec2()),
                ));
            }
        }
    }
    return neighbours;
}
