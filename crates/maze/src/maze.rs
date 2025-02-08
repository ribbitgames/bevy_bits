use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use bevy::prelude::*;
use bitflags::bitflags;
use lazy_static::lazy_static;
use rand::Rng;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct CollisionFlags: u32 {
        const NOCOL = 0x00000000;
        const COL_U = 0x00000001;
        const COL_D = 0x00000002;
        const COL_L = 0x00000004;
        const COL_R = 0x00000008;
    }
}

impl CollisionFlags {
    pub const fn count_ones(self) -> u32 {
        self.bits().count_ones()
    }
}

//const DIRECTIONS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
const DIRECTIONS: [IVec2; 4] = [
    IVec2::new(1, 0),
    IVec2::new(0, 1),
    IVec2::new(-1, 0),
    IVec2::new(0, -1),
];

lazy_static! {
    static ref DIRS: HashMap<CollisionFlags, IVec2> = {
        let mut m = HashMap::new();
        m.insert(CollisionFlags::COL_U, IVec2::new(0, -1));
        m.insert(CollisionFlags::COL_D, IVec2::new(0, 1));
        m.insert(CollisionFlags::COL_L, IVec2::new(-1, 0));
        m.insert(CollisionFlags::COL_R, IVec2::new(1, 0));
        m
    };
}

#[derive(Component)]
pub struct MazeGenerator {
    w: usize,
    h: usize,
    tiles: Vec<CollisionFlags>,
}

impl MazeGenerator {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            tiles: vec![CollisionFlags::NOCOL; w * h],
        }
    }
    pub const fn height(&self) -> usize {
        self.h
    }
    pub const fn width(&self) -> usize {
        self.w
    }
    const fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.w
    }
    const fn raw_index(&self, x: usize, y: usize) -> usize {
        x + y * (self.w * 2 + 1)
    }
    pub fn generate(&mut self) {
        let raw_w = self.w * 2 + 1;
        let raw_h = self.h * 2 + 1;
        let mut raw_tiles = vec![true; raw_w * raw_h];
        for h in (1..raw_h).step_by(2) {
            for w in (1..raw_w).step_by(2) {
                self.grow_maze(w as i32, h as i32, &mut raw_tiles);
            }
        }
        for h in 0..raw_h {
            for w in 0..raw_w {
                if let Some(tile) = raw_tiles.get(self.raw_index(w, h)) {
                    print!("{}", if *tile { "*" } else { " " });
                }
            }
            println!();
        }
        self.compress(raw_tiles);
    }
    fn compress(&mut self, raw_tiles: Vec<bool>) {
        let raw_w = self.w * 2 + 1;
        let raw_h = self.h * 2 + 1;
        for h in (1..raw_h).step_by(2) {
            for w in (1..raw_w).step_by(2) {
                let mut col = CollisionFlags::NOCOL;
                if let Some(tile) = raw_tiles.get(self.raw_index(w, h - 1)) {
                    if *tile {
                        col |= CollisionFlags::COL_U;
                    }
                }
                if let Some(tile) = raw_tiles.get(self.raw_index(w, h + 1)) {
                    if *tile {
                        col |= CollisionFlags::COL_D;
                    }
                }
                if let Some(tile) = raw_tiles.get(self.raw_index(w - 1, h)) {
                    if *tile {
                        col |= CollisionFlags::COL_L;
                    }
                }
                if let Some(tile) = raw_tiles.get(self.raw_index(w + 1, h)) {
                    if *tile {
                        col |= CollisionFlags::COL_R;
                    }
                }
                let index = self.index((w - 1) / 2, (h - 1) / 2);
                if let Some(tile) = self.tiles.get_mut(index) {
                    *tile = col;
                }
            }
        }
    }
    pub fn can_go(&self, src: IVec2, dir: IVec2) -> bool {
        if let Some(tile) = self.tiles.get(self.index(src.x as usize, src.y as usize)) {
            return match (dir.x, dir.y) {
                (1, 0) => *tile & CollisionFlags::COL_R != CollisionFlags::COL_R,
                (0, 1) => *tile & CollisionFlags::COL_D != CollisionFlags::COL_D,
                (-1, 0) => *tile & CollisionFlags::COL_L != CollisionFlags::COL_L,
                (0, -1) => *tile & CollisionFlags::COL_U != CollisionFlags::COL_U,
                _ => false,
            };
        }
        false
    }
    fn grow_maze(&self, x: i32, y: i32, raw_tiles: &mut [bool]) {
        let mut last_dir = IVec2::ZERO;
        if let Some(tile) = raw_tiles.get_mut(self.raw_index(x as usize, y as usize)) {
            *tile = false;
        };
        let mut cells: Vec<IVec2> = vec![IVec2::new(x, y)];
        while !cells.is_empty() {
            if let Some(cell) = cells.last() {
                let mut unmade_cells: Vec<IVec2> = Vec::new();
                /*
                for (dx, dy) in DIRECTIONS {
                    if self.can_dig(cell, IVec2::new(dx, dy), raw_tiles) {
                        unmade_cells.push(IVec2::new(dx, dy));
                    }
                }*/
                for direc in DIRECTIONS {
                    if self.can_dig(*cell, direc, raw_tiles) {
                        unmade_cells.push(direc);
                    }
                }
                if !unmade_cells.is_empty() {
                    let mut rng = rand::rng();
                    let dir = if unmade_cells.contains(&last_dir) && rng.random_range(0. ..1.) > 0.5 {
                        last_dir
                    } else {
                        *(unmade_cells
                            .get(rng.random_range(0..unmade_cells.len()))
                            .expect("The index is out of the range, something wrong"))
                    };
                    let dst = *cell + dir;
                    let dst2 = *cell + dir * 2;
                    if let Some(tile) =
                        raw_tiles.get_mut(self.raw_index(dst.x as usize, dst.y as usize))
                    {
                        *tile = false;
                    }
                    if let Some(tile) =
                        raw_tiles.get_mut(self.raw_index(dst2.x as usize, dst2.y as usize))
                    {
                        *tile = false;
                    }
                    cells.push(dst2);
                    last_dir = dir;
                } else {
                    cells.remove(cells.len() - 1);
                    last_dir = IVec2::ZERO;
                }
            }
        }
    }
    fn can_dig(&self, p: IVec2, d: IVec2, raw_tiles: &[bool]) -> bool {
        let dst = p + d * 3;
        let sx = self.w as i32 * 2 + 1;
        let sy = self.h as i32 * 2 + 1;
        if dst.x < 0 || dst.x >= sx || dst.y < 0 || dst.y >= sy {
            return false;
        }
        let dst = p + d * 2;
        if let Some(tile) = raw_tiles.get(self.raw_index(dst.x as usize, dst.y as usize)) {
            return *tile;
        }
        false
    }
    fn is_deadend(&self, p: IVec2) -> bool {
        if let Some(tile) = self.tiles.get(self.index(p.x as usize, p.y as usize)) {
            return tile.count_ones() == 3;
        }
        false
    }
    pub fn get_deadends(&self) -> Vec<IVec2> {
        let mut deadends: Vec<IVec2> = vec![];
        for y in 0..self.h {
            for x in 0..self.w {
                if self.is_deadend(IVec2::new(x as i32, y as i32)) {
                    deadends.push(IVec2::new(x as i32, y as i32));
                }
            }
        }
        deadends
    }
}

impl Display for MazeGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(tile) = self.tiles.get(self.index(x, y)) {
                    write!(f, "{:>04b} ", *tile)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
