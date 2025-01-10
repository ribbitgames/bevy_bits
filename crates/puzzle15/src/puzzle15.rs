use std::fmt::{self, Display, Formatter};

use bevy::prelude::*;
use rand::Rng;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Panel {
    Empty,
    PanelNumber(u8),
}

#[derive(Component)]
pub struct PuzzlePanels {
    w: usize,
    h: usize,
    panels: Vec<Panel>,
}

impl PuzzlePanels {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            panels: Self::reset_panels(w, h),
        }
    }
    pub fn reset(&mut self) {
        self.panels = Self::reset_panels(self.w, self.h);
    }
    pub const fn width(&self) -> usize {
        self.w
    }
    pub const fn height(&self) -> usize {
        self.h
    }
    pub fn get_index(&self, i: i32) -> IVec2 {
        let idx = self
            .panels
            .iter()
            .position(|&r| r == Panel::PanelNumber(i as u8))
            .expect("") as i32;
        IVec2::new(idx % self.w as i32, idx / self.w as i32)
    }
    pub fn get_panel(&self, x: usize, y: usize) -> Option<&Panel> {
        self.panels.get(self.index(x, y))
    }
    fn reset_panels(w: usize, h: usize) -> Vec<Panel> {
        (0..w * h)
            .map(|i| {
                if i < w * h - 1 {
                    Panel::PanelNumber((i + 1) as u8)
                } else {
                    Panel::Empty
                }
            })
            .collect()
    }
    const fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.w
    }
    pub fn slide(&mut self, pos: IVec2, dir: IVec2) {
        let mut panels_index_to_move: Vec<usize> = Vec::new();
        panels_index_to_move.push(self.index(pos.x as usize, pos.y as usize));
        if self.slide_sub(pos, dir, &mut panels_index_to_move) {
            panels_index_to_move.reverse();
            for panel_index_to_move in panels_index_to_move {
                if let Some(panel) = self.panels.get(panel_index_to_move) {
                    let panel_to_move = *panel;
                    let index = self.index(
                        ((panel_index_to_move % self.w) as i32 + dir.x) as usize,
                        ((panel_index_to_move / self.w) as i32 + dir.y) as usize,
                    );
                    if let Some(panel_dst) = self.panels.get_mut(index) {
                        *panel_dst = panel_to_move;
                    }
                }
            }
            let index = self.index(pos.x as usize, pos.y as usize);
            if let Some(panel) = self.panels.get_mut(index) {
                *panel = Panel::Empty;
            }
        }
    }
    fn slide_sub(&self, pos: IVec2, dir: IVec2, panels_index_to_move: &mut Vec<usize>) -> bool {
        let next_pos = pos + dir;
        if next_pos.x < 0
            || next_pos.y < 0
            || next_pos.x >= self.w as i32
            || next_pos.y >= self.h as i32
        {
            return false;
        }
        if self
            .panels
            .get(self.index(next_pos.x as usize, next_pos.y as usize))
            == Some(&Panel::Empty)
        {
            return true;
        }
        panels_index_to_move.push(self.index(next_pos.x as usize, next_pos.y as usize));
        self.slide_sub(next_pos, dir, panels_index_to_move)
    }
    fn get_empty_index(&self) -> Option<usize> {
        for y in 0..self.h {
            for x in 0..self.w {
                let index = self.index(x, y);
                if let Some(id) = self.panels.get(index) {
                    if *id == Panel::Empty {
                        return Some(index);
                    }
                }
            }
        }
        None
    }
    pub fn slide_random(&mut self, interation_count: usize) {
        if let Some(idx) = self.get_empty_index() {
            let mut x = idx % self.w;
            let mut y = idx / self.w;
            let mut rng = rand::thread_rng();
            for i in 0..interation_count {
                let dir = if i % 2 == 0 {
                    let mut xx = rng.gen_range(0..self.w - 1);
                    xx = if x == xx { xx + 1 } else { xx };
                    std::mem::swap(&mut x, &mut xx);
                    IVec2::new((xx as i32 - x as i32).signum(), 0)
                } else {
                    let mut yy = rng.gen_range(0..self.h - 1);
                    yy = if y == yy { yy + 1 } else { yy };
                    std::mem::swap(&mut y, &mut yy);
                    IVec2::new(0, (yy as i32 - y as i32).signum())
                };
                println!("slide from {} to {}", IVec2::new(x as i32, y as i32), dir);
                self.slide(IVec2::new(x as i32, y as i32), dir);
            }
        }
    }
    pub fn is_solved(&self) -> bool {
        for i in 0..self.w * self.h - 1 {
            if let Some(id) = self.panels.get(i) {
                if *id != Panel::PanelNumber((i + 1) as u8) {
                    return false;
                }
            }
        }
        true
    }
}

impl Display for PuzzlePanels {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..self.h {
            for x in 0..self.w {
                match self.panels.get(self.index(x, y)) {
                    Some(Panel::Empty) => {
                        write!(f, "   ",)?;
                    }
                    Some(Panel::PanelNumber(v)) => {
                        write!(f, "{v:>02} ")?;
                    }
                    None => {}
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
