// Simple QuadTree
// Alexis Andre (@mactuitui)

use nannou::prelude::*;

pub struct QuadTree {
    indices: [usize; 4],
    num: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub children: Option<[Box<QuadTree>; 4]>,
}

pub trait WithPos {
    fn get_pos(&self) -> Vec2;
}

impl QuadTree {
    pub fn new() -> Self {
        QuadTree {
            indices: [0; 4],
            num: 0,
            x: 0.0,
            y: 0.0,
            width: 1024.0,
            height: 1024.0,
            children: None,
        }
    }

    pub fn contains<T: WithPos>(&self, point: &T) -> bool {
        let pos = point.get_pos();
        pos.x >= self.x - self.width * 0.5
            && pos.x < self.x + self.width * 0.5
            && pos.y >= self.y - self.height * 0.5
            && pos.y < self.y + self.height * 0.5
    }

    //split in four
    //mutates self
    pub fn split<T: WithPos>(&mut self, elements: &[T]) {
        //let new_children =
        let mut southwest: Box<QuadTree> = Box::new(QuadTree {
            indices: [0; 4],
            num: 0,
            x: self.x - self.width * 0.25,
            y: self.y - self.height * 0.25,
            width: self.width * 0.5,
            height: self.height * 0.5,
            children: None,
        });
        let mut southeast: Box<QuadTree> = Box::new(QuadTree {
            indices: [0; 4],
            num: 0,
            x: self.x + self.width * 0.25,
            y: self.y - self.height * 0.25,
            width: self.width * 0.5,
            height: self.height * 0.5,
            children: None,
        });
        let mut northwest: Box<QuadTree> = Box::new(QuadTree {
            indices: [0; 4],
            num: 0,
            x: self.x - self.width * 0.25,
            y: self.y + self.height * 0.25,
            width: self.width * 0.5,
            height: self.height * 0.5,
            children: None,
        });
        let mut northeast: Box<QuadTree> = Box::new(QuadTree {
            indices: [0; 4],
            num: 0,
            x: self.x + self.width * 0.25,
            y: self.y + self.height * 0.25,
            width: self.width * 0.5,
            height: self.height * 0.5,
            children: None,
        });
        //push the elements down the children
        for k in 0..self.num {
            let point = &elements[self.indices[k]];
            if northwest.contains(point) {
                northwest.insert(elements, self.indices[k]);
            } else if northeast.contains(point) {
                northeast.insert(elements, self.indices[k]);
            } else if southeast.contains(point) {
                southeast.insert(elements, self.indices[k]);
            } else if southwest.contains(point) {
                southwest.insert(elements, self.indices[k]);
            }
        }
        self.children = Some([northwest, northeast, southwest, southeast]);
    }

    pub fn insert_children<T: WithPos>(&mut self, elements: &[T], index: usize) {
        match self.children.as_mut() {
            Some(children) => {
                //TODO faster sort
                if children[0].contains(&elements[index]) {
                    children[0].insert(elements, index);
                } else if children[1].contains(&elements[index]) {
                    children[1].insert(elements, index);
                } else if children[2].contains(&elements[index]) {
                    children[2].insert(elements, index);
                } else if children[3].contains(&elements[index]) {
                    children[3].insert(elements, index);
                }
            }
            None => {}
        }
    }

    pub fn intersects(&self, x: f32, y: f32, dist: f32) -> bool {
        //think of the inverse, if one rect is completely on the side of another
        if x + dist < self.x - self.width * 0.5 || x - dist > self.x + self.width * 0.5 {
            return false;
        }
        if y + dist < self.y - self.height * 0.5 || y - dist > self.y + self.height * 0.5 {
            return false;
        }

        return true;
    }

    pub fn get_elements<T: WithPos>(
        &self,
        elements: &[T],
        x: f32,
        y: f32,
        dist: f32,
    ) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();

        //are we intersecting the rect
        if self.intersects(x, y, dist) {
            //if we have children, recurse
            match &self.children {
                Some(children) => {
                    let mut r0 = children[0].get_elements(elements, x, y, dist);
                    let mut r1 = children[1].get_elements(elements, x, y, dist);
                    let mut r2 = children[2].get_elements(elements, x, y, dist);
                    let mut r3 = children[3].get_elements(elements, x, y, dist);
                    result.append(&mut r0);
                    result.append(&mut r1);
                    result.append(&mut r2);
                    result.append(&mut r3);
                }
                None => {
                    //no children, return all elements?
                    for i in 0..self.num {
                        result.push(self.indices[i]);
                    }
                }
            }
        }
        result
    }

    pub fn insert<T: WithPos>(&mut self, elements: &[T], index: usize) {
        //have we split?
        match &self.children {
            Some(_children) => {
                self.insert_children(elements, index);
            }
            None => {
                if self.num < 4 {
                    self.indices[self.num] = index;
                    self.num += 1;
                } else {
                    //we are full
                    // we must split and push down the elements
                    self.split(elements);
                    //add this to children
                    self.insert_children(elements, index);
                }
            }
        }
    }
}
