use super::*;

pub mod input {
    use super::*;
    //I don't like this, use set_grid_cell directly?
    pub fn handle_mouse_input<T:GraphNode>(camera:&Camera, graph:&mut SparseDirectedGraph<T>, location:&mut Location, color: usize, height: u32) {
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = camera.screen_to_world(Vec2::from(mouse_position()));
            let new_pointer = ExternalPointer::new(Index(color), height);
            if let Some(pointer) = set_grid_cell(new_pointer, mouse_pos, *location, graph) {
                location.pointer = pointer;
            }
        }
    }

    use editing::*;
    mod editing {
        use super::*;
        pub fn set_grid_cell<T : GraphNode + std::hash::Hash + Eq>(to:ExternalPointer, world_point:Vec2, location:Location, graph:&mut SparseDirectedGraph<T>) -> Option<ExternalPointer> {
            let height = to.height;
            if height <= location.pointer.height {
                let cell = gate::point_to_cells(location, height, world_point)[0];
                if let Some(cell) = cell {
                    let path = ZorderPath::from_cell(cell, location.pointer.height - height);
                    if let Ok(pointer) = graph.set_node(location.pointer, &path.steps(), to.pointer) {
                        return Some(pointer)
                    } else {dbg!("Write failure. That's really bad.");}
                }
            }
            None
        }

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
        
        pub fn draw_all<T:GraphNode>(camera:&Camera, graph:&SparseDirectedGraph<T>, entities:&EntityPool, blocks:&BlockPalette) {
            let mut locations_to_draw = Vec::new();
            for entity in entities.entities.iter() {
                let location = entity.location;
                if camera.aabb.intersects(bounds::aabb(location.position, location.pointer.height)) == BVec2::TRUE {
                    locations_to_draw.push(location.clone());
                }
            }
            for location in locations_to_draw {
                draw(camera, graph, &location, blocks);
            }
        }
    
        pub fn draw<T:GraphNode>(camera:&Camera, graph:&SparseDirectedGraph<T>, location:&Location, blocks:&BlockPalette) {
            let grid_length = bounds::cell_length(location.pointer.height);
            let grid_top_left = location.position - grid_length / 2.;
            camera.outline_vec_rectangle(
                grid_top_left,
                grid_length,
                0.03,
                WHITE
            );
            let object_top_left = location.position - grid_length / 2.;
            let leaves = graph.dfs_leave_cells(location.pointer);
            for leaf in leaves {
                let color = blocks.blocks[*leaf.pointer.pointer].color; 
                let cell_top_left = object_top_left + bounds::top_left_corner(leaf.cell, leaf.pointer.height);
                if 0 != *leaf.pointer.pointer{
                    camera.draw_vec_rectangle(
                    cell_top_left,
                    bounds::cell_length(leaf.pointer.height),
                    color
                    );
                }
                camera.outline_vec_rectangle(
                cell_top_left,
                bounds::cell_length(leaf.pointer.height),
                0.03,
                WHITE
                );
            }
        }
    }

}


pub struct Camera { 
    pub aabb : AABB,
    scale_zoom: f32,
    zoom:f32,
    screen_percentage: f32,
}
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
        self.outline_bounds(self.aabb, 2., WHITE);
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

    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        let p1 = self.world_to_screen(point1);
        let p2 = self.world_to_screen(point2);
        draw_line(p1.x, p1.y, p2.x, p2.y, line_width*self.zoom(), color);
    }

    pub fn outline_bounds(&self, bounds:AABB, line_width:f32, color:Color) {
        self.outline_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), line_width, color);
    } 

}
