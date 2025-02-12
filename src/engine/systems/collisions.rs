use std::cmp::{Reverse, Ordering};
use std::collections::BinaryHeap;

use super::*;

#[derive(Debug, Clone, new)]
pub struct CollisionObject {
    pub position : Vec2, //Grid Center
    pub velocity : Vec2,
    pub angular_velocity : f32,
    pub owner : ID,
    pub hitting : ID,
    pub particles : BinaryHeap<Reverse<Particle>>,
}

#[derive(Debug, Clone, new)]
pub struct Particle {
    pub offset : Vec2,
    #[new(value = "0.")]
    pub ticks_into_projection : f32,
    pub position_data : [Option<CellData>; 4],
    pub corner_type : CornerType,
    #[new(value = "0")]
    itt_counter : usize,
}
impl PartialOrd for Particle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Particle {
    fn cmp(&self, other: &Self) -> Ordering { 
        if self.ticks_into_projection.approx_eq(other.ticks_into_projection) { Ordering::Equal }
        else if self.ticks_into_projection.less(other.ticks_into_projection) { Ordering::Less }
        else if self.ticks_into_projection.greater(other.ticks_into_projection) { Ordering::Greater }
        else { unreachable!() }
    } 
}
impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool { self.ticks_into_projection.approx_eq(other.ticks_into_projection) }
}
impl Eq for Particle {} 
impl Particle {
    fn position(&self, owner:&CollisionObject) -> Vec2 {
        self.offset + owner.position + owner.velocity * self.ticks_into_projection
    }
    fn tick(&mut self, delta_tick: f32, angular_velocity:f32) {
        self.ticks_into_projection = (self.ticks_into_projection + delta_tick).snap_zero();
        self.offset = self.offset.rotate(Vec2::from_angle(angular_velocity * delta_tick));
        self.corner_type = self.corner_type.rotate(angular_velocity * delta_tick);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CornerType {
    TopLeft,
    Top(f32),
    TopRight,
    Right(f32),
    BottomRight,
    Bottom(f32),
    BottomLeft,
    Left(f32),
}
impl CornerType {
    // Replace this clamped logic with .less and .greater methods
    pub fn checks(&self, velocity:Vec2) -> CheckZorders {
        if velocity.is_zero() { panic!("AHHH (Velocity isn't non_zero)"); }
        let clamped = velocity.zero_signum().max(IVec2::ZERO);
        if velocity.x.is_zero() {
            match self {
                Self::Top(_) | Self::Bottom(_) => CheckZorders::Two([2 * clamped.y as usize, (2 * clamped.y as usize) | 1]),
                Self::TopLeft | Self::BottomLeft | Self::Left(_) => CheckZorders::One((2 * clamped.y as usize) | 1),
                Self::TopRight | Self::BottomRight | Self::Right(_) => CheckZorders::One(2 * clamped.y as usize),
            }
        } else if velocity.y.is_zero() {
            match self {
                Self::Left(_) | Self::Right(_) => CheckZorders::Two([2 | clamped.x as usize, clamped.x as usize]),
                Self::TopLeft | Self::TopRight | Self::Top(_) => CheckZorders::One(2 | clamped.x as usize),
                Self::BottomLeft | Self::BottomRight | Self::Bottom(_) => CheckZorders::One(clamped.x as usize),
            }
        } else { CheckZorders::One(clamped.x as usize | (2 * clamped.y as usize)) }
    }
    pub fn hittable_walls(&self, velocity:Vec2) -> BVec2 {
        BVec2::from_array(match self {
            Self::TopLeft => [velocity.x.less(0.), velocity.y.less(0.)],
            Self::TopRight => [velocity.x.greater(0.), velocity.y.less(0.)],
            Self::BottomLeft => [velocity.x.less(0.), velocity.y.greater(0.)],
            Self::BottomRight => [velocity.x.greater(0.), velocity.y.greater(0.)],
            Self::Top(_) => [!velocity.x.is_zero(), velocity.y.less(0.)],
            Self::Bottom(_) => [!velocity.x.is_zero(), velocity.y.greater(0.)],
            Self::Left(_) => [velocity.x.less(0.), !velocity.y.is_zero()],
            Self::Right(_) => [velocity.x.greater(0.), !velocity.y.is_zero()],
        })
    }
    
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::TopLeft,
            1 => Self::TopRight,
            2 => Self::BottomLeft,
            3 => Self::BottomRight,
            _ => unimplemented!()
        }
    }

    pub fn rotation(&self) -> f32 {
        match self {
            Self::BottomRight => PI/4.,
            Self::BottomLeft => PI * 3./4.,
            Self::TopLeft => PI * 5./4.,
            Self::TopRight => PI * 7./4.,
            Self::Top(angle) 
            | Self::Left(angle) 
            | Self::Right(angle) 
            | Self::Bottom(angle) => *angle,
        }
    }
    pub fn from_rotation(rotation: f32) -> Self {
        match rotation.normalize_angle() {
            rot if rot.angle_approx_eq(PI / 4.) => Self::BottomRight,
            rot if rot.angle_approx_eq(PI * 3./4.) => Self::BottomLeft,
            rot if rot.angle_approx_eq(PI * 5./4.) => Self::TopLeft,
            rot if rot.angle_approx_eq(PI * 7./4.) => Self::TopRight,
            rot if rot.less(PI / 4.) => Self::Right(rot),
            rot if rot.less(PI * 3./4.) => Self::Bottom(rot),
            rot if rot.less(PI * 5./4.) => Self::Left(rot),
            rot if rot.less(PI * 7./4.) => Self::Top(rot),
            rot => Self::Right(rot),
        }
    }
    pub fn rotate(&self, rotation: f32) -> Self {
        Self::from_rotation(self.rotation() + rotation)
    }

}

pub enum CheckZorders {
    One(usize),
    Two([usize; 2]),
}
impl CheckZorders {
    pub fn from_velocity(velocity: Vec2) -> Self {
        match velocity.zero_signum() {
            IVec2 { x: 0, y: -1 } => CheckZorders::Two([0, 1]),  // Up: check top cells
            IVec2 { x: 0, y: 1 } => CheckZorders::Two([2, 3]),   // Down: check bottom cells
            IVec2 { x: -1, y: 0 } => CheckZorders::Two([0, 2]), // Left: check left cells
            IVec2 { x: 1, y: 0 } => CheckZorders::Two([1, 3]),  // Right: check right cells
            IVec2 { x: 0, y: 0 } => panic!("Ahhh! (Velocity is zero)"),
            _ => CheckZorders::One((velocity.y.greater(0.) as usize) * 2 + (velocity.x.greater(0.) as usize)),
        }
    }
}

struct Hit {
    pub owner : ID,
    pub hitting : ID,
    // Relative Normal of the hit
    pub walls : BVec2,
    pub ticks : f32,
}

// Eventually turn this into an island identifier/generator
fn collect_collision_objects() -> Vec<CollisionObject> {
    let mut objects = Vec::new();
    let entities = GAME_STATE.entities.read();
    for idx in 0..entities.entities.len() {
        let entity = &entities.entities[idx];
        for other_idx in idx + 1..entities.entities.len() {
            let other = &entities.entities[other_idx];
            // if within_range(&entity, &other) {
                if let Some(obj) = entity_to_collision_object(entity, other) { 
                    objects.push(obj); 
                }
                if let Some(obj) = entity_to_collision_object(other, entity) { 
                    objects.push(obj); 
                }
            // }
        }
    }
    objects
}

fn apply_drag() {
    const DRAG_MULTIPLIER: f32 = 0.95;
    for entity in &mut GAME_STATE.entities.write().entities { 
        entity.velocity = (entity.velocity * DRAG_MULTIPLIER).snap_zero();
        entity.angular_velocity = (entity.angular_velocity * DRAG_MULTIPLIER).snap_zero();
    }
}

fn tick_entities(delta_tick: f32) {
    for entity in &mut GAME_STATE.entities.write().entities {
        entity.location.position += (entity.velocity * delta_tick).snap_zero();
        entity.rel_rotate(entity.angular_velocity * delta_tick);
    }
}

fn apply_normal_force(static_thing: ID, hit: Hit) {
    let mut entities = GAME_STATE.entities.write();
    let hitting = entities.get_entity(hit.hitting).unwrap();
    // // I want this guy, but the precision errors are strong with him
    // let world_normal = hit.walls.to_vec2().rotate(hitting.forward);
    // let rel_velocity = entities.get_entity(hit.owner).unwrap().velocity - hitting.velocity;
    // let world_impulse = (rel_velocity.dot(world_normal) * world_normal).snap_zero();
    
    let world_to_hitting = Mat2::from_angle(-hitting.rotation);
    let relative_velocity = world_to_hitting.mul_vec2(
        entities.get_entity(hit.owner).unwrap().velocity - hitting.velocity
    );
    let relative_normals = hit.walls.as_vec2();
    let world_impulse = world_to_hitting.transpose()
        .mul_vec2(relative_velocity * relative_normals)
        .snap_zero();

    let objects = [(hit.owner, -1.0), (hit.hitting, 1.0)];
    for (id, multiplier) in objects {
        if id != static_thing {
            let entity = entities.get_mut_entity(id).unwrap();
            entity.velocity = (entity.velocity + world_impulse * multiplier).snap_zero();
            entity.angular_velocity = 0.;
        }
    }
}

pub async fn n_body_collisions(static_thing: ID) {
    let mut tick_max = 1.;
    let mut wedge_count = 0;
    loop {
        let objects = collect_collision_objects();
        let Some(mut hit) = find_next_action(objects, tick_max).await else {
            tick_entities(tick_max); break
        };
        if hit.ticks.is_zero() {
            wedge_count += 1;
            // if wedge_count == 2: Balance the velocity of the axis we double tap
            // ^ Only relevant when we have elastic collisions
            if wedge_count == 3 { hit.walls = BVec2::TRUE }
        } else {
            wedge_count = 0;
            tick_max = tick_max - hit.ticks;
            tick_entities(hit.ticks);
        }
        apply_normal_force(static_thing, hit);
    }
    apply_drag();
}

// Eventually make this work with islands, solving each island by itself
async fn find_next_action(objects:Vec<CollisionObject>, tick_max:f32) -> Option<Hit> {
    let itt_cut = 30;
    let entities = GAME_STATE.entities.read();
    let mut ticks_to_action = tick_max;
    let mut action = None;
    'objectloop : for mut object in objects {
        while let Some(Reverse(mut cur_corner)) = object.particles.pop() {
            cur_corner.itt_counter += 1;
            GAME_STATE.camera.read().outline_point(cur_corner.position(&object), 0.05, 0.01, Color {
                r: (cur_corner.itt_counter as f32) / itt_cut as f32,
                g: (cur_corner.itt_counter as f32) / itt_cut as f32,
                b: (cur_corner.itt_counter as f32) / itt_cut as f32,
                a: 1.,
            });
            if cur_corner.itt_counter >= itt_cut {
                if cur_corner.itt_counter == itt_cut { macroquad::window::next_frame().await }
                dbg!("Too many iterations"); 
                // continue 
            }
            if cur_corner.ticks_into_projection.greater_eq(ticks_to_action) { continue 'objectloop }
            let hitting_location = entities.get_entity(object.hitting).unwrap().location;
            let Some(ticks_to_hit) = next_intersection(
                &cur_corner,
                &object,
                hitting_location,
                ticks_to_action,
            ) else { continue };
            cur_corner.tick(ticks_to_hit, object.angular_velocity);
            let position_data = gate::point_to_real_cells(
                hitting_location,
                cur_corner.position(&object)
            );
            cur_corner.position_data = position_data;
            let real_velocity = object.velocity + angular_to_tangential_velocity(
                object.angular_velocity,
                cur_corner.offset
            );
            if let Some(walls_hit) = hitting_wall(cur_corner.position_data, real_velocity, cur_corner.corner_type) {
                action = Some( Hit {
                        owner : object.owner,
                        hitting : object.hitting,
                        walls : walls_hit,
                        ticks : cur_corner.ticks_into_projection
                    }
                );
                ticks_to_action = cur_corner.ticks_into_projection;
            } else { object.particles.push(Reverse(cur_corner)) }
        }
    }
    action
}

// Selects the appropriate cell and height based on position data and indices
fn select_cell_and_height(position_data: &[Option<CellData>; 4], col_zorders: CheckZorders) -> Option<(Vec2, u32)> {
    Some(match col_zorders {
        CheckZorders::Two(indices) => {
            match indices.into_iter().filter_map(|index| position_data[index]).map(|data| data.bound_data()).collect::<Vec<_>>().as_slice() {
                [cell] => *cell,
                [cell1, cell2] => if cell1.1 < cell2.1 { *cell1 } else { *cell2 },
                _ => None?
            }
        },
        CheckZorders::One(idx) => position_data[idx]?.bound_data()
    })
}

fn next_intersection(
    particle: &Particle,
    object: &CollisionObject,
    hitting_location: Location,
    tick_max: f32,
) -> Option<f32> {
    let point = particle.position(object);
    let point_velocity = object.velocity + angular_to_tangential_velocity(
        object.angular_velocity,
        particle.offset
    );
    let hitting_aabb = bounds::aabb(hitting_location.position, hitting_location.pointer.height);
    let top_left = hitting_aabb.min();
    let within_bounds = hitting_aabb.contains(point);
    if hitting_wall(particle.position_data, point_velocity, particle.corner_type).is_some() {
        // hitting_wall(position_data, velocity, corner_type);
        return Some(0.)
    }
    let (cell, height) = if within_bounds != BVec2::TRUE {
        (hitting_aabb.exterior_will_intersect(point, point_velocity)?, hitting_location.pointer.height)
    } else { select_cell_and_height(&particle.position_data, CheckZorders::from_velocity(point_velocity))? };

    let quadrant = point_velocity.signum().max(Vec2::ZERO);
    let cell_length = bounds::cell_length(height);
    let boundary_corner = top_left + (cell + quadrant) * cell_length;
    let mut ticks  = Vec2::splat(f32::INFINITY);
    let motion = Motion {
        center_of_rotation : object.position + object.velocity * particle.ticks_into_projection,
        velocity : object.velocity,
        offset : particle.offset,
        angular_velocity : object.angular_velocity,
    };
    if let Some(tickx) = motion.solve_line_intersection(
        Line::Vertical(boundary_corner.x), 
        tick_max
    ) { ticks.x = tickx }
    if let Some(ticky) = motion.solve_line_intersection(
        Line::Horizontal(boundary_corner.y), 
        tick_max.min(ticks.x.abs())
    ) { 
        if ticky.abs().less(ticks.y.abs()) { ticks.y = ticky } 
    }

    let ticks_to_hit = match (within_bounds.x, within_bounds.y) {
        (false, false) => ticks.max_element(),
        (true, false) if ticks.x.is_zero() => ticks.y,
        (false, true) if ticks.y.is_zero() => ticks.x,
        _ => ticks.min_element(),
    };

    (!ticks_to_hit.is_nan() && ticks_to_hit.abs().less_eq(tick_max)).then_some(ticks_to_hit) 
}

// Make this work again
// pub fn within_range(entity1:&Entity, entity2:&Entity) -> bool {
//     let aabb = bounds::aabb(entity1.location.position, entity1.location.pointer.height).expand(entity1.velocity);
//     let aabb2 = bounds::aabb(entity2.location.position, entity2.location.pointer.height).expand(entity2.velocity);
//     let result = aabb.intersects(aabb2) == BVec2::TRUE;
//     let color = if result { GREEN } else { RED };
//     let camera = GAME_STATE.camera.read();
//     camera.outline_bounds(aabb, 0.05, color);
//     camera.outline_bounds(aabb2, 0.05, color);
//     result
// }

// Add culling for when no rotation?
pub fn entity_to_collision_object(owner:&Entity, hitting:&Entity) -> Option<CollisionObject> {
    let mut collision_points = BinaryHeap::new();
    let align_to_hitting = Vec2::from_angle(-hitting.rotation);
    let offset = bounds::center_to_edge(owner.location.pointer.height);
    let rel_angular = (owner.angular_velocity - hitting.angular_velocity).snap_zero();
    let rel_velocity = ((owner.velocity - hitting.velocity).rotate(align_to_hitting)).snap_zero();
    if rel_velocity.is_zero() && rel_angular.is_zero() { return None }
    let rotated_owner_pos = (owner.location.position - hitting.location.position).rotate(align_to_hitting) + hitting.location.position;
    let camera = GAME_STATE.camera.read();
    camera.draw_point(rotated_owner_pos, 0.1, GREEN);
    let point_rotation = align_to_hitting.rotate(owner.forward);
    for corners in owner.corners.iter() {
        for i in 0..4 {
            //Cull any corner which isn't exposed
            if corners.mask & (1 << i) == 0 { continue }
            let point = (corners.points[i] - offset).rotate(point_rotation);
            let corner_type = CornerType::from_index(i).rotate(owner.rotation - hitting.rotation);
            let color = match corner_type {
                CornerType::TopLeft => LIME,
                CornerType::TopRight => BLUE,
                CornerType::BottomLeft => RED,
                CornerType::BottomRight => YELLOW,
                CornerType::Top(_) => PURPLE,
                CornerType::Bottom(_) => WHITE,
                CornerType::Left(_) => GRAY,
                CornerType::Right(_) => DARKGRAY,
            };
            camera.draw_point(point + rotated_owner_pos, 0.1, color);
            collision_points.push(Reverse(Particle::new(
                point,
                gate::point_to_real_cells(
                    hitting.location,
                    point + rotated_owner_pos
                ),
                corner_type
            )));
        }
    }
    Some(CollisionObject::new(
        rotated_owner_pos,
        rel_velocity,
        rel_angular,
        owner.id,
        hitting.id,
        collision_points
    ))
}

#[derive(Debug, Clone, new)]
pub struct Corners {
    pub points : [Vec2; 4],
    pub index : Index,
    pub mask : u8,
}

pub mod corner_handling {
    use super::*;

    //Figure out if this can be improved?
    fn cell_corner_mask(start:ExternalPointer, zorder:ZorderPath) -> u8 {
            let mut exposed_mask = 0b1111;
            let checks = [
                (IVec2::new(-1, 0), 0b01), //Top Left 0
                (IVec2::new(0, -1), 0b10),
                (IVec2::new(-1, -1), 0b11),
                (IVec2::new(1, 0), 0b00), //Top Right 1
                (IVec2::new(0, -1), 0b11),
                (IVec2::new(1, -1), 0b10),
                (IVec2::new(-1, 0), 0b11), //Bottom Left 2
                (IVec2::new(0, 1), 0b00),
                (IVec2::new(-1, 1), 0b01),
                (IVec2::new(1, 0), 0b10), //Bottom Right 3
                (IVec2::new(0, 1), 0b01),
                (IVec2::new(1, 1), 0b00),
            ];
            for i in 0 .. 4 {
                for j in 0 .. 3 {
                    let (offset, direction) = checks[i*3 + j];
                    let Some(mut check_zorder) = zorder.move_cartesianly(offset) else { continue };
                    for _ in 0 .. start.height - check_zorder.depth {
                        check_zorder = check_zorder.step_down(direction)
                    }
                    let pointer = GAME_STATE.graph.read().read(start, &check_zorder.steps()).unwrap();
                    if GAME_STATE.blocks.is_solid_index(*pointer.pointer) { exposed_mask -= 1 << i; break }
                }
            }
            exposed_mask
        }

    //The top left corner of the root is (0, 0)
    fn cell_corners(cell:CellData) -> [Vec2; 4] {
        let cell_size = bounds::cell_length(cell.pointer.height);
        let top_left_corner = cell.cell.as_vec2() * cell_size;
        [
            top_left_corner,
            top_left_corner.with_x(top_left_corner.x + cell_size.x),
            top_left_corner.with_y(top_left_corner.y + cell_size.y),
            top_left_corner + cell_size,
        ]
    }

    pub fn tree_corners(start:ExternalPointer) -> Vec<Corners> {
        let leaves = GAME_STATE.graph.read().dfs_leaf_cells(start);
        let mut corners = Vec::new();
        for cell in leaves {
            let zorder = ZorderPath::from_cell(cell.cell, start.height - cell.pointer.height);
            corners.push( Corners::new(
                cell_corners(cell),
                cell.pointer.pointer,
                if !GAME_STATE.blocks.is_solid_index(*cell.pointer.pointer) { 0 } else { cell_corner_mask(start, zorder) }
            ));
        }
        corners 
    }
    
    
}

fn hitting_wall(position_data:[Option<CellData>; 4], velocity:Vec2, corner_type:CornerType) -> Option<BVec2> {
    let mut hit_walls = corner_type.hittable_walls(velocity);
    // Velocity Check 
    {
        let hit = match corner_type.checks(velocity) {
            CheckZorders::One(index) => GAME_STATE.blocks.is_solid_cell(position_data[index]),
            CheckZorders::Two([idx1, idx2]) => GAME_STATE.blocks.is_solid_cell(position_data[idx1]) | GAME_STATE.blocks.is_solid_cell(position_data[idx2]),
        };
        if !velocity.x.is_zero() { hit_walls.x &= hit }
        if !velocity.y.is_zero() { hit_walls.y &= hit }
    };
    'slide_check: {
        if hit_walls != BVec2::TRUE { break 'slide_check } 
        let idxs = match velocity.zero_signum() {
            IVec2{x: -1, y: -1} => [2, 1],
            IVec2{x: -1, y: 1} => [0, 3],
            IVec2{x: 1, y: -1} => [3, 0],
            IVec2{x: 1, y: 1} => [1, 2],
            _ => unreachable!(),
        };
        let slide = BVec2::new(
            GAME_STATE.blocks.is_solid_cell(position_data[idxs[0]]),
            GAME_STATE.blocks.is_solid_cell(position_data[idxs[1]])
        );

        if slide != BVec2::FALSE { hit_walls &= slide }
    };
    (hit_walls != BVec2::FALSE).then_some(hit_walls)
}
