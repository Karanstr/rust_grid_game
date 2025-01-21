use super::*;

pub mod input {
    use super::*;
    use editing::*;
    mod editing {
        use super::*;
        

        /*
        pub fn expand_object_domain(&mut self, object_index:usize, direction:usize) {
            let object = &mut self.objects[object_index];
            //Prevent zorder overflow for now
            if object.root.height == 15 { dbg!("We don't overflow around here"); return }
            object.position += object.cell_length(0) * zorder_to_direction(direction as u32)/2.;
            let new_root = self.graph.set_node(NodePointer::new(Index(0)), &[direction as u32], object.root.pointer).unwrap();
            self.graph.swap_root(object.root.pointer, new_root);
            object.root.pointer = new_root;
            object.root.height += 1;
        }

        pub fn shrink_object_domain(&mut self, object_index:usize, preserve_direction:usize) {
            let object = &mut self.objects[object_index];
            if object.root.height == 0 { return }
            object.position += object.cell_length(0) * -zorder_to_direction(preserve_direction as u32)/4.;
            let new_root = self.graph.set_node(object.root.pointer, &[], self.graph.child(object.root.pointer, preserve_direction).unwrap()).unwrap();
            self.graph.swap_root(object.root.pointer, new_root);
            object.root.pointer = new_root;
            object.root.height -= 1;
        }*/
    }

}

pub mod output {
    use super::*;

    pub mod render {
        use super::*;
        
        pub fn draw_all(camera:&Camera, entities:&EntityPool, blocks:&BlockPalette) {
            for entity in entities.entities.iter() {
                let location = entity.location;
                if camera.aabb.intersects(bounds::aabb(location.position, location.pointer.height)) == BVec2::TRUE {
                    draw(camera, entity, blocks);
                }
            }
        }
    
        pub fn draw(camera:&Camera, entity:&Entity, blocks:&BlockPalette) {
            let angle = entity.forward;
            for cell in entity.cells.iter() {
                if *cell.pointer.pointer != 0 {
                    let color = blocks.blocks[*cell.pointer.pointer as usize].color;
                    let rotated_points:Vec<Vec2> = cell.corners.iter().map(|(point, _)| {
                        Vec2::new(
                            point.x * angle.x - point.y * angle.y,
                            point.x * angle.y + point.y * angle.x
                        ) + entity.location.position
                    }).collect();

                    camera.draw_rectangle_from_corners(&rotated_points, color);
                }
            }
        }
    }
}

use macroquad::shapes::{draw_rectangle, draw_rectangle_lines, draw_line, draw_triangle};
use macroquad::miniquad::window::screen_size;
pub struct Camera { 
    pub aabb : AABB,
    scale_zoom: f32,
    zoom:f32,
    screen_percentage: f32,
}
#[allow(dead_code)]
impl Camera {
    pub fn new(aabb:AABB, screen_percentage:f32) -> Self {
        let scale_zoom = (Vec2::from(screen_size()) * screen_percentage).min_element() / (2. * aabb.radius().min_element());
        Self { 
            aabb, 
            scale_zoom,
            zoom: 1.,
            screen_percentage
        }
    }

    pub fn update(&mut self, new_position:Vec2, smoothing:f32) {
        self.lerp_position(new_position, smoothing);
        self.scale_zoom = (Vec2::from(screen_size())*self.screen_percentage).min_element() / (2. * self.aabb.radius().min_element());
    }

    pub fn change_zoom(&mut self, zoom:f32) { self.zoom *= zoom }

    pub fn change_screen_percentage(&mut self, screen_percentage:f32) {
        self.screen_percentage = screen_percentage;
        self.update(self.aabb.center(), 0.);
    }

    fn zoom(&self) -> f32 { self.zoom * self.scale_zoom }

    pub fn show_view(&self) {
        self.outline_bounds(self.aabb, 0.05, WHITE);
    }

    fn lerp_position(&mut self, position:Vec2, smoothing:f32) {
        self.aabb.move_to(self.aabb.center().lerp(position, smoothing));
    }
 
}
impl Camera {
    fn global_offset(&self) -> Vec2 {
        self.aabb.center() - Vec2::from(screen_size()) / 2. / self.zoom()
    }

    pub fn world_to_screen(&self, world_position:Vec2) -> Vec2 {
       (world_position - self.global_offset()) * self.zoom()
    }

    pub fn screen_to_world(&self, screen_position:Vec2) -> Vec2 {
        screen_position / self.zoom() + self.global_offset()
    }
}
impl Camera {
    #[allow(dead_code)]
    pub fn draw_vec_rectangle(&self, position:Vec2, length:Vec2, color:Color) {
        let pos = self.world_to_screen(position);
        let len = length * self.zoom();
        draw_rectangle(pos.x, pos.y, len.x, len.y, color);
    }

    pub fn outline_vec_rectangle(&self, position:Vec2, length:Vec2, line_width:f32, color:Color) {
        let pos = self.world_to_screen(position);
        let len = length * self.zoom();
        draw_rectangle_lines(pos.x, pos.y, len.x, len.y, line_width*self.zoom(), color);
    }
    
    #[allow(dead_code)]
    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        let p1 = self.world_to_screen(point1);
        let p2 = self.world_to_screen(point2);
        draw_line(p1.x, p1.y, p2.x, p2.y, line_width*self.zoom(), color);
    }

    pub fn outline_bounds(&self, bounds:AABB, line_width:f32, color:Color) {
        self.outline_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), line_width, color);
    } 

    pub fn draw_rectangle_from_corners(&self, corners:&[Vec2], color: Color) {
        let corners:Vec<Vec2> = corners.iter().map(|point| self.world_to_screen(*point)).collect();
        draw_triangle(
            corners[0],
            corners[1],
            corners[2],
            color
        );
        draw_triangle(
            corners[1],
            corners[2],
            corners[3],
            color
        );
    }

}
