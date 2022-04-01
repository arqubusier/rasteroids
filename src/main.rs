extern crate sdl2;
 

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use ndarray::arr2;
use ndarray::arr1;
use ndarray::Array2;
use ndarray::Array1;
use rand::Rng;
use std::time::Duration;
use std::f64::consts::PI;

 

type FVector = Array1<f64>;
type FMatrix = Array2<f64>;

struct Entity {
    position: FVector,
    velocity: FVector,
    acceleration: f64,
    angle: f64,
    angle_speed: f64,
    collision_radius: f64
}

trait GameObject {
    fn shift(&mut self, world_x: f64, world_y: f64);
}


impl GameObject for Entity {
    fn shift(&mut self, world_x: f64, world_y: f64) {
	move_entity(self, world_x, world_y);
    }
}

struct Shot {
    entity: Entity,
    time_to_live: u64
}

impl GameObject for Shot {
    fn shift(&mut self, world_x: f64, world_y: f64) {
	move_entity(&mut self.entity, world_x, world_y);
    }
}


fn translator(deltaX: f64, deltaY: f64) -> FMatrix {
    arr2(&[[1.0, 0.0, 0.0],
           [0.0, 1.0, 0.0],
           [deltaX, deltaY, 1.0]])
}

fn rotator(angle: f64) -> FMatrix {
    arr2(&[[angle.cos(), -angle.sin(), 0.0],
           [angle.sin(), angle.cos(), 0.0],
           [0.0, 0.0, 1.0]])
}
 

fn draw_polygon(polygon: &FMatrix, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    let n = polygon.shape()[0];
    for i in 0..(n-1) {
	let p1 = Point::new(polygon[[i, 0]] as i32, polygon[[i, 1]] as i32);
	let p2 = Point::new(polygon[[i+1, 0]] as i32, polygon[[i+1, 1]] as i32);
        canvas.draw_line(p1, p2);
    }

    let p1 = Point::new(polygon[[n-1, 0]] as i32, polygon[[n-1, 1]] as i32);
    let p2 = Point::new(polygon[[0, 0]] as i32, polygon[[0, 1]] as i32);
    canvas.draw_line(p1, p2);
}

fn move_entity(entity: &mut Entity, world_x: f64, world_y: f64) {
    entity.angle += entity.angle_speed;
    let acceleration_vector =  arr1(&[0.0 ,entity.acceleration, 1.0]).dot(&rotator(entity.angle));
    entity.velocity = &entity.velocity + &acceleration_vector;
    entity.position = &entity.position + &entity.velocity;

    if entity.position[0] > world_x {
	entity.position[0] -= world_x
    }
    if entity.position[0] < 0.0 {
	entity.position[0] += world_x
    }
    if entity.position[1] > world_y {
	entity.position[1] -= world_y
    }
    if entity.position[1] < 0.0 {
	entity.position[1] += world_y
    }
}
 
fn move_game_objects<T: GameObject>(objects: &mut [T], world_x: f64, world_y: f64) {
    for object in objects.iter_mut() {
	object.shift(world_x, world_y);
    }
}

fn points_ship(ship: &Entity) -> FMatrix {
    let w = 7.0;
    let h = 10.0;
    let points = arr2(&[[0.0, h, 1.0],
                        [-w, -h, 1.0],
                        [w, -h, 1.0]]);


    points.dot(&rotator(ship.angle)).dot(&translator(ship.position[0], ship.position[1]))
}

fn points_asteroid(asteroid: &Entity, original_radius: f64) -> FMatrix {
    let r = 20.0;
    // Add last column afterwards to prevent its scaling
    let ones = arr2(&[[0.0, 0.0, 1.0],
                      [0.0, 0.0, 1.0],
                      [0.0, 0.0, 1.0],
                      [0.0, 0.0, 1.0],
                      [0.0, 0.0, 1.0],
                      [0.0, 0.0, 1.0],
                      [0.0, 0.0, 1.0],
                      [0.0, 0.0, 1.0],
                      [0.0, 0.0, 1.0]]);

    let points = arr2(&[[0.0, r, 0.0],
                        [3.0, r-2.0, 0.0],
                        [r, 0.0, 0.0],
                        [10.0, -r, 0.0],
                        [1.0, -10.0, 0.0],
                        [3.0, -r, 0.0],
                        [-15.0, -15.0, 0.0],
                        [-10.0, 0.0, 0.0],
                        [-14.0, 11.0, 0.0]]);

    let scaled_points = (asteroid.collision_radius/original_radius)*points + ones;
    //let scaled_points = points;
    scaled_points.dot(&rotator(asteroid.angle)).dot(&translator(asteroid.position[0], asteroid.position[1]))
}

fn points_shot(shot: &Shot) -> FMatrix {
    let l = shot.entity.collision_radius;
    let points = arr2(&[[0.0,  l, 1.0],
			[0.0, -l, 1.0]]);
    points.dot(&rotator(shot.entity.angle)).dot(&translator(shot.entity.position[0], shot.entity.position[1]))
}

fn new_asteroid(position: &FVector, velocity: &FVector, collision_radius: f64) -> Entity {
    let mut rng = rand::thread_rng();
    let new_angle: f64 = rng.gen_range(0.01..1.0);
    let new_angle_speed: f64 = rng.gen_range(-0.2..0.2);

    Entity {position: position.clone(), velocity: velocity.clone(),
	    acceleration: 0.0, angle: new_angle, angle_speed: new_angle_speed,
	    collision_radius: collision_radius}
}

fn split_asteroid(asteroid: &Entity) -> Vec<Entity> {
    let mut asteroids: Vec<Entity> = Vec::new();
    if asteroid.collision_radius >= 9.0 {
	let new_radius = asteroid.collision_radius / 2.0;
	for i in 0..4 {
	  asteroids.push(new_asteroid(&asteroid.position, &arr1(&[0.0 ,0.2, 1.0]).dot(&rotator(2.0*PI/i as f64)), new_radius));
	}
    }

    asteroids
}

fn add_shot(ship: &Entity, shots: &mut Vec<Shot>) {
    let shot_speed = 10.0;
    let position =  ship.position.clone() + arr1(&[0.0, ship.collision_radius+5.0, 1.0]).dot(&rotator(ship.angle));
    let velocity =  ship.velocity.clone() + arr1(&[0.0, shot_speed, 1.0]).dot(&rotator(ship.angle));
    shots.push(Shot{ entity: Entity {position: position.clone(), velocity: velocity.clone(),
			     acceleration: 0.0, angle: ship.angle, angle_speed: 0.0,
			     collision_radius: 5.0}, time_to_live: 60 } );
}

fn is_collided(e1: &Entity, e2: &Entity, world_x: f64, world_y: f64) -> bool {
    fn is_intersected(p1: &FVector, p2: &FVector, collision_radius1: f64, collision_radius2: f64) -> bool {
	let delta_x = (p1[0] - p2[0]).abs();
	let delta_y = (p1[1] - p2[1]).abs();
	let dist_pow = delta_x.powi(2) + delta_y.powi(2);

	dist_pow < (collision_radius1.powi(2) + collision_radius2.powi(2))
    }

    fn wrapped_position(e: &Entity, world_x: f64, world_y: f64) -> FVector {
	let mut res = e.position.clone();
	if e.position[0] - e.collision_radius  > world_x {
	    res[0] -= world_x;
	}
	if e.position[1] - e.collision_radius  > world_y {
	    res[0] -= world_y;
	}

	res
    }

    is_intersected(&wrapped_position(e1, world_x, world_y), &wrapped_position(e2, world_x, world_y),
		   e1.collision_radius, e2.collision_radius)
}

fn starting_asteroids(n: u64, starting_zone: Entity, world_x: f64, world_y: f64) -> Vec<Entity> {
    let mut rng = rand::thread_rng();
    let mut i: u64 = 0;
    let mut asteroids: Vec<Entity> =  Vec::new();

    loop {
	if i >= n {
	    break;
	}
	let new_x: f64 = rng.gen_range(0.0..world_x as f64);
	let new_y: f64 = rng.gen_range(0.0..world_y as f64);
	let new_velocity_x: f64 = rng.gen_range(0.0..1.0);
	let new_velocity_y: f64 = rng.gen_range(0.0..1.0);
	let position = arr1(&[new_x, new_y, 1.0]);
	let velocity = arr1(&[new_velocity_x, new_velocity_y, 1.0]);
	let mut asteroid = new_asteroid(&position, &velocity, 20.0);

	if !is_collided(&starting_zone, &asteroid, world_x, world_y) {
	    asteroids.push(asteroid); 
	    i += 1;
	}
    }

    asteroids
}
 
pub fn main() {
    fn handle_event(event: &Event, ship: &mut Entity, shots: &mut Vec<Shot>) -> bool {
	let acceleration = 0.10;
	let turn_speed = 0.1;

	let mut quit: bool = false;
	    match event {
		Event::Quit {..} |
		Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
		    quit = true;
		},
		Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. } => {
		    ship.angle_speed += turn_speed;
		},
		Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. } => {
		    ship.angle_speed -= turn_speed;
		},
		Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, .. } => {
		    ship.acceleration += acceleration;
		},
		Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, .. } => {
		    ship.acceleration -= acceleration;
		},
		Event::KeyDown { keycode: Some(Keycode::Space), repeat: false, .. } => {
		    add_shot(&ship, shots);
		},
		Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
		    ship.angle_speed = 0.0;
		},
		Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
		    ship.angle_speed = 0.0;
		},
		Event::KeyUp { keycode: Some(Keycode::Up), .. } => {
		    ship.acceleration = 0.0;
		},
		Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
		    ship.acceleration = 0.0;
		},
		_ => {}
	    }

	quit
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let world_x: f64 = 800.0;
    let world_y: f64 = 600.0;
    let start_x: f64 = world_x/2.0;
    let start_y: f64 = world_y/2.0;
    let window = video_subsystem.window("rust-sdl2 demo", world_x as u32, world_y as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;

    let mut ship = Entity { position: arr1(&[start_x, start_y, 1.0]),
			    velocity: arr1(&[0.0, 0.0, 0.0]),
			    angle: 1.0, acceleration: 0.0, angle_speed: 0.0,
			    collision_radius: 15.0 };
    let asteroid_start_radius = 20.0;
    let starting_zone = Entity { position: arr1(&[start_x, start_y, 1.0]),
				velocity: arr1(&[0.0, 0.0, 0.0]),
				angle: 1.0, acceleration: 0.0, angle_speed: 0.0,
				collision_radius: 50.0 };
    let mut asteroids: Vec<Entity> = starting_asteroids(10, starting_zone, world_x, world_y);
    let mut shots: Vec<Shot> = Vec::new();
    let v = arr1(&[200.0,200.0,1.0]);

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for event in event_pump.poll_iter() {
	    if handle_event(&event, &mut ship, &mut shots) {
		break 'running;
	    }
        }

	for shot in shots.iter_mut() {
	    shot.time_to_live -= 1;
	}
	shots.retain(|shot| {
	    shot.time_to_live > 0
	});
	ship.shift(world_x, world_y);
	move_game_objects(&mut asteroids, world_x, world_y);
	move_game_objects(&mut shots, world_x, world_y);

	for asteroid in asteroids.iter() {
	    if is_collided(&ship, &asteroid, world_x, world_y) {
		break 'running;
	    }
	}
	let mut new_asteroids : Vec<Entity> = Vec::new();
	asteroids.retain(|asteroid| {
	    match shots.iter().position(|shot| {is_collided(&asteroid, &shot.entity, world_x, world_y)}) {
		None => true,
		Some(index) => {
		    new_asteroids.append(&mut split_asteroid(asteroid));
		    shots.swap_remove(index);
		    false
		}
	    }
	});
	asteroids.append(&mut new_asteroids);
	

        canvas.set_draw_color(Color::RGB(255,255,255));
        draw_polygon(&points_ship(&ship), &mut canvas);
	for asteroid in asteroids.iter() {
	    draw_polygon(&points_asteroid(&asteroid, asteroid_start_radius), &mut canvas);
	}
	for shot in shots.iter() {
	    draw_polygon(&points_shot(&shot), &mut canvas);
	}
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

}
