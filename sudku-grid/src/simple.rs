#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub use rand;

use rand::{distributions::{Distribution, Uniform}, Rng};
use std::fmt;
use std::ops::{Index, IndexMut};
use std::mem::swap;

type Pos = (usize, usize);

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum GridLayout {
    #[default]
    Row,
    Col,
    Box,
}

pub type Nums3x3 = [[u8; 9]; 9];

pub const BASE_3X3: Nums3x3 = [
    [1, 2, 3, 4, 5, 6, 7, 8, 9],
    [4, 5, 6, 7, 8, 9, 1, 2, 3],
    [7, 8, 9, 1, 2, 3, 4, 5, 6],
    [2, 3, 4, 5, 6, 7, 8, 9, 1],
    [5, 6, 7, 8, 9, 1, 2, 3, 4],
    [8, 9, 1, 2, 3, 4, 5, 6, 7],
    [3, 4, 5, 6, 7, 8, 9, 1, 2],
    [6, 7, 8, 9, 1, 2, 3, 4, 5],
    [9, 1, 2, 3, 4, 5, 6, 7, 8],
];

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Grid3x3([[u8; 9]; 9], GridLayout);

impl Grid3x3 {
    pub const fn new() -> Self {
        Self(BASE_3X3, GridLayout::Row)
    }

    pub const fn empty() -> Self {
        Self([[0u8; 9]; 9], GridLayout::Row)
    }

    pub fn randomized() -> Self {
        let mut grid = Self::new();
        grid.randomize();
        grid
    }

    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        /*
         * 1 = Swap two rows w/n 3x3 border
         * 2 = Same with cols
         * 3 = Swap two groups (of 3) of rows
         * 4 = Same with cols
         * 5 = Reflect on y = x
         * 6 = Reflect on y = -x (transpose)
         * 7 = Rotate 90
         * 8 = Rotate 180
         * 9 = Rotate 270
         * 10 = Reflect on x-axis
         * 11 = Reflect on y-axis
         */
        let transforms = Uniform::from(1..=11);
        let of_nine = Uniform::from(0..8);
        let num_transforms = rng.gen_range(10..=100);
        for _ in 0..num_transforms {
            match transforms.sample(&mut rng) {
                t @ 1..=2 => {
                    if t == 1 {
                        self.make_row_wise();
                    } else {
                        self.make_col_wise();
                    }
                    let row1 = of_nine.sample(&mut rng);
                    let row1m = row1 % 3;
                    let row2 = loop {
                        let row2m = of_nine.sample(&mut rng) % 3;
                        if row1m != row2m {
                            // Get the row2 number from the row 1 number
                            break row1 / 3 * 3 + row2m;
                        }
                    };
                    self.0.swap(row1, row2);
                    self.make_row_wise();
                },
                t @ 3..=4 => {
                    if t == 3 {
                        self.make_row_wise();
                    } else {
                        self.make_col_wise();
                    }
                    let group1 = of_nine.sample(&mut rng) % 3;
                    let group2 = loop {
                        let g2 = of_nine.sample(&mut rng) % 3;
                        if g2 != group1 {
                            break g2;
                        }
                    };
                    for i in 0..3 {
                        self.0.swap(group1 * 3 + i, group2 * 3 + i);
                    }
                    self.make_row_wise();
                }
                5 => self.reflect_y_x(),
                6 => self.reflect_y_neg_x(),
                7 => self.rotate_90(),
                8 => self.rotate_180(),
                9 => self.rotate_270(),
                10 => self.reflect_x(),
                11 => self.reflect_y(),
                _ => unreachable!(),
            }
        }
        // Sanity check
        // TODO: Possibly panic
        if let Some(pos) = self.is_valid() {
            self.randomize();
        }
    }

    // n is the number to remove
    pub fn remove_nums(&mut self, n: usize) {
        assert!(n < 81, "cannot remove more symbols than exists");
        // At least n^2 - 1 distict symbols must be kept when removing symbols to have a unique
        // solution (having n^2 - 1 does not mean the solution is unique, though).
        // Source: https://pi.math.cornell.edu/~mec/Summer2009/Mahmood/More.html

        // Keeps track of whether the one number that can be fully removed has been
        let mut num_gone = false;
        // Allocate 10 instad of 9 so that the nums from the squares can be converted from their
        // values to a usize without having to subtract 1 to get the index and worry about zeros
        let mut nums = [9u8; 10];
        let mut rng = rand::thread_rng();
        let coords = Uniform::from(0..9);
        for _ in 0..n {
            let mut removed = false;
            while !removed {
                let mut x = coords.sample(&mut rng);
                let mut y = coords.sample(&mut rng);
                for _ in 0..2 {
                    let num = self[y][x] as usize;
                    if num != 0 && (!num_gone || nums[num] != 1) {
                        self[y][x] = 0;
                        nums[num] -= 1;
                        removed = true;
                        if nums[num] == 0 {
                            num_gone = true;
                        }
                        break;
                    }
                    // Try this so that we don't have to roll RNG again
                    // TODO: Other transformations like 8 - x, 8 - y, (y, x) etc.
                    (x, y) = (8 - x, 8 - y);
                }
            }
        }
    }

    // Returns true if the number can be placed in the square
    pub fn pos_is_valid(&self, pos: Pos, n: u8) -> bool {
        if n == 0 {
            return true;
        }
        let mut arr = [0u8; 9];

        self.get_row_for(pos, &mut arr);
        if arr.contains(&n) {
            return false;
        }
        self.get_col_for(pos, &mut arr);
        if arr.contains(&n) {
            return false;
        }
        self.get_box_for(pos, &mut arr);
        !arr.contains(&n)
    }

    // Returns None if valid, otherwise, returns the position of the first bad square encountered.
    // The board must be filled for it to be valid in this case.
    // Theory:
    // When you have checked the columns and rows, you can skip some boxes. Having checked the top
    // 3 rows and boxes 1 and 2, you don't need to check box 3. Likewise you don't need to check
    // box 6 if you have 4 and 5, and the bottom boxes are proven correct by having checked the
    // columns and the first six boxes. The last row is proven correct by the bottom 3 boxes and
    // the two rows above it. So you can skip 1 row and 5 boxes for 21 checks total.
    // Source: https://puzzling.stackexchange.com/questions/26118/minimum-steps-to-verify-a-sudoku-solution
    // MathOverflow Source: https://mathoverflow.net/questions/129143/verifying-the-correctness-of-a-sudoku-solution

    // TODO: Try to optimize better
    pub fn is_valid(&self) -> Option<Pos> {
        let mut arr = [0u8; 9];
        // Don't assign to silence "unused_assignments" warning
        let mut check;
        // Check the first 8 rows and columns
        for y in 0..8 {
            // Skip boxes 3, 6, 7, and 8
            if y != 2 && y < 5 {
                check = 0;
                // (y * 3) % 9 gets the x of the first square (top left) of a box
                self.get_box_for(((y * 3) % 9, y), &mut arr);
                for i in 0..9 {
                    let n = arr[i] as u16;
                    let num = 1 << n;
                    if check & num != 0 {
                        return Some((i % 3, i / 3));
                    }
                    check |= num;
                }
            }
            check = 0;
            // Check row
            for x in 0..9 {
                let n = self[y][x];
                let num = 1 << n;
                if check & num != 0 {
                    return Some((x, y));
                }
                check |= num;
            }
            // Check col
            check = 0;
            let x = y;
            for y in 0..9 {
                let n = self[y][x];
                let num = 1 << n;
                if check & num != 0 {
                    return Some((x, y));
                }
                check |= num;
            }
        }
        // Check last col
        check = 0;
        for y in 0..9 {
            let n = self[y][8];
            let num = 1 << n;
            if check & num != 0 {
                return Some((8, y));
            }
            check |= num;
        }
        None
    }

    pub fn get_row_for(&self, (_, y): Pos, arr: &mut [u8; 9]) {
        for x in 0..9 {
            arr[x] = self[y][x];
        }
    }

    pub fn get_col_for(&self, (x, _): Pos, arr: &mut [u8; 9]) {
        for y in 0..9 {
            arr[y] = self[y][x];
        }
    }

    pub fn get_box_for(&self, (x, y): Pos, arr: &mut [u8; 9]) {
        let (x, y) = (x / 3 * 3, y / 3 * 3);
        let mut i = 0;
        for y in y..y + 3 {
            for x in x..x + 3 {
                arr[i] = self[y][x];
                i += 1;
            }
        }
    }

    fn make_row_wise(&mut self) {
        match self.1 {
            GridLayout::Row => return,
            GridLayout::Col => self.reflect_y_x(),
            GridLayout::Box => self.make_box_from_row(),
        }
        self.1 = GridLayout::Row;
    }

    fn make_col_wise(&mut self) {
        match self.1 {
            GridLayout::Col => return,
            GridLayout::Row => self.reflect_y_x(),
            GridLayout::Box => self.make_box_from_col(),
        }
        self.1 = GridLayout::Col;
    }

    fn make_box_wise(&mut self) {
        match self.1 {
            GridLayout::Box => return,
            GridLayout::Row => self.make_box_from_row(),
            GridLayout::Col => self.make_box_from_col(),
        }
        self.1 = GridLayout::Box;
    }

    fn make_box_from_row(&mut self) {
        for i in 0..3 {
            let (mut y1, mut y2) = (i * 3 + 1, i * 3);
            for x in 0..3 {
                self.swap_pos((x, y1), (x + 3, y2));
            }
            y1 = i * 3 + 2;
            for x in 0..3 {
                self.swap_pos((x, y1), (x + 6, y2));
            }
            y2 = i * 3 + 1;
            for x in 4..6 {
                self.swap_pos((x, y1), (x + 3, y2));
            }
        }
    }

    fn make_box_from_col(&mut self) {
        for i in 0..3 {
            let (mut x1, mut x2) = (i * 3 + 1, i * 3);
            for y in 0..3 {
                self.swap_pos((x1, y), (x2, y + 3));
            }
            x1 = i * 3 + 2;
            for y in 0..3 {
                self.swap_pos((x1, y), (x2, y + 6));
            }
            x2 = i * 3 + 1;
            for y in 4..6 {
                self.swap_pos((x1, y), (x2, y + 3));
            }
        }
    }

    // Reflects on the line y = x
    #[inline]
    fn reflect_y_x(&mut self) {
        for y in 0..9 {
            for x in y + 1..9 {
                self.swap_pos((x, y), (y, x));
            }
        }
    }

    // Reflects on the line y = -x (i.e., transpose)
    #[inline]
    fn reflect_y_neg_x(&mut self) {
        for y in 0..9 {
            // (0..9 - y - 1)
            for x in (0..8 - y).rev() {
                self.swap_pos((x, y), (8 - y, 8 - x));
            }
        }
    }

    // Source: https://www.geeksforgeeks.org/rotate-a-matrix-by-90-degree-in-clockwise-direction-without-using-any-extra-space/
    fn rotate_90(&mut self) {
        // 0..9 / 2
        for y in 0..4 {
            // (0..9 - y - 1)
            for x in y..8 - y {
                let temp = self[y][x];
                // All 8s are just 9 - 1 condensed
                self[y][x] = self[8 - x][y];
                self[8 - x][y] = self[8 - y][8 - x];
                self[8 - y][8 - x] = self[x][8 - y];
                self[x][8 - y] = temp;
            }
        }
    }

    fn rotate_180(&mut self) {
        for y in 0..4 {
            for x in 0..9 {
                self.swap_pos((x, y), (8 - x, 8 - y));
            }
        }
        // Swap the middle row since there's an odd number of rows
        for x in 0..4 {
            self[4].swap(x, 8 - x);
        }
    }

    // Source: https://www.enjoyalgorithms.com/blog/rotate-a-matrix-by-90-degrees-in-an-anticlockwise-direction
    fn rotate_270(&mut self) {
        for y in 0..4 {
            // (0..9 - y - 1)
            for x in y..8 - y {
                let temp = self[y][x];
                // All 8s are just 9 - 1 condensed
                self[y][x] = self[x][8 - y];
                self[x][8 - y] = self[8 - y][8 - x];
                self[8 - y][8 - x] = self[8 - x][y];
                self[8 - x][y] = temp;
            }
        }
    }

    fn reflect_x(&mut self) {
        for y in 0..4 {
            self.0.swap(y, 8 - y);
        }
    }

    fn reflect_y(&mut self) {
        for y in 0..9 {
            for x in 0..4 {
                self[y].swap(x, 8 - x);
            }
        }
    }

    #[inline]
    fn swap_pos(&mut self, (x1, y1): Pos, (x2, y2): Pos) {
        // Make sure they aren't the same position
        if x1 != x2 || y1 != y2 {
            unsafe {
                // Rust doesn't allow the second mutable borrow even though they
                // individual elements referenced won't share any memory at all.
                let r1 = &mut self.0[y1][x1] as *mut u8;
                let r2 = &mut self.0[y2][x2] as *mut u8;
                swap(&mut *r1, &mut *r2);
            }
        }
    }
}

impl Index<Pos> for Grid3x3 {
    type Output = u8;
    fn index(&self, (x, y): Pos) -> &Self::Output {
        &self.0[y][x]
    }
}

impl IndexMut<Pos> for Grid3x3 {
    fn index_mut(&mut self, (x, y): Pos) -> &mut Self::Output {
        &mut self.0[y][x]
    }
}

impl Index<usize> for Grid3x3 {
    type Output = [u8; 9];
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Grid3x3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl fmt::Display for Grid3x3 {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..9 {
            for x in 0..9 {
                let c = if self.0[y][x] == 0 { '_' } else { (self.0[y][x] + b'0') as char };
                match x {
                    2 | 5 => write!(w, "{} | ", c)?,
                    8 => write!(w, "{}", c)?,
                    _ => write!(w, "{} ", c)?,
                }
            }
            if y % 3 == 2 && y != 8 {
                write!(w, "\n---------------------\n")?;
            } else {
                write!(w, "\n")?;
            }
        }
        Ok(())
    }
}

pub type Nums4x4 = [[u8; 16]; 16];

pub const BASE_4X4: Nums4x4 = [
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4],
    [9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8],
    [13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1],
    [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5],
    [10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    [14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
    [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2],
    [7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6],
    [11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    [15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
    [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3],
    [8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7],
    [12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    [16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
];

#[derive(Default)]
pub struct Grid4x4([[u8; 16]; 16], GridLayout);

impl Grid4x4 {
    pub const fn new() -> Self {
        Self(BASE_4X4, GridLayout::Row)
    }

    pub fn empty() -> Self {
        Self([[0u8; 16]; 16], GridLayout::Row)
    }

    pub fn randomized() -> Self {
        let mut grid = Self::new();
        grid.randomize();
        grid
    }

    pub fn randomize(&mut self) {
        use rand::{distributions::{Distribution, Uniform}, Rng};

        assert!(self.is_valid().is_none());

        let mut rng = rand::thread_rng();
        /*
         * 1 = Swap two rows w/n 4x4 border
         * 2 = Same with cols
         * 3 = Swap two groups (of 4) of rows
         * 4 = Same with cols
         * 5 = Reflect on y = x
         * 6 = Reflect on y = -x (transpose)
         * 7 = Rotate 90
         * 8 = Rotate 180
         * 9 = Rotate 270
         * 10 = Reflect on x-axis
         * 11 = Reflect on y-axis
         */
        let transforms = Uniform::from(1..=6);
        let of_sixteen = Uniform::from(0..16);
        let num_transforms = rng.gen_range(10..=100);
        for _ in 0..num_transforms {
            match transforms.sample(&mut rng) {
                t @ 1..=2 => {
                    if t == 1 {
                        self.make_row_wise();
                    } else {
                        self.make_col_wise();
                    }
                    let row1 = of_sixteen.sample(&mut rng);
                    let row1m = row1 % 3;
                    let row2 = loop {
                        let row2m = of_sixteen.sample(&mut rng) % 4;
                        if row1m != row2m {
                            // Get the row2 number from the row 1 number
                            break row1 / 4 * 4 + row2m;
                        }
                    };
                    self.0.swap(row1, row2);
                    self.make_row_wise();
                },
                t @ 3..=4 => {
                    if t == 3 {
                        self.make_row_wise();
                    } else {
                        self.make_col_wise();
                    }
                    let group1 = of_sixteen.sample(&mut rng) % 4;
                    let group2 = loop {
                        let g2 = of_sixteen.sample(&mut rng) % 4;
                        if g2 != group1 {
                            break g2;
                        }
                    };
                    for i in 0..4 {
                        self.0.swap(group1 * 4 + i, group2 * 4 + i);
                    }
                    self.make_row_wise();
                }
                5 => self.reflect_y_x(),
                6 => self.reflect_y_neg_x(),
                7 => self.rotate_90(),
                8 => self.rotate_180(),
                9 => self.rotate_270(),
                10 => self.reflect_x(),
                11 => self.reflect_y(),
                _ => unreachable!(),
            }
        }
        // Sanity check
        // TODO: Possibly panic
        if let Some(pos) = self.is_valid() {
            self.randomize();
        }
    }

    // n is the number to remove
    pub fn remove_nums(&mut self, n: usize) {
        assert!(n < 256, "cannot remove more symbols than exists");
        // At least n^2 - 1 distict symbols must be kept when removing symbols to have a unique
        // solution (having n^2 - 1 does not mean the solution is unique, though).
        // Source: https://pi.math.cornell.edu/~mec/Summer2009/Mahmood/More.html

        // Keeps track of whether the one number that can be fully removed has been
        let mut num_gone = false;
        // Allocate 17 instad of 16 so that the nums from the squares can be converted from their
        // values to a usize without having to subtract 1 to get the index and worry about zeros
        let mut nums = [16u8; 17];
        let mut rng = rand::thread_rng();
        let coords = Uniform::from(0..16);
        for _ in 0..n {
            let mut removed = false;
            while !removed {
                let mut x = coords.sample(&mut rng);
                let mut y = coords.sample(&mut rng);
                for _ in 0..2 {
                    let num = self[y][x] as usize;
                    if num != 0 && (!num_gone || nums[num] != 1) {
                        self[y][x] = 0;
                        nums[num] -= 1;
                        removed = true;
                        if nums[num] == 0 {
                            num_gone = true;
                        }
                        break;
                    }
                    // Try this so that we don't have to roll RNG again
                    // TODO: Other transformations like 8 - x, 8 - y, (y, x) etc.
                    (x, y) = (15 - x, 15 - y);
                }
            }
        }
    }

    // Returns true if the number can be placed in the square
    pub fn pos_is_valid(&self, pos: Pos, n: u8) -> bool {
        if n == 0 {
            return true;
        }
        let mut arr = [0u8; 16];

        self.get_row_for(pos, &mut arr);
        if arr.contains(&n) {
            return false;
        }
        self.get_col_for(pos, &mut arr);
        if arr.contains(&n) {
            return false;
        }
        self.get_box_for(pos, &mut arr);
        !arr.contains(&n)
    }

    // Returns None if valid, otherwise, returns the position of the first bad square encountered.
    // TODO: Try to optimize better
    pub fn is_valid(&self) -> Option<Pos> {
        let mut arr = [0u8; 16];
        // Don't assign to silence "unused_assignments" warning
        let mut check;
        // Check the first 8 rows and columns
        for y in 0..15 {
            // Skip boxes 4, 8, 12, 14, 14, 15, 16
            if y != 4 && y != 7 && y < 11 {
                check = 0;
                // (y * 4) % 16 gets the x of the first square (top left) of a box
                self.get_box_for(((y * 4) % 16, y), &mut arr);
                for i in 0..16 {
                    let n = arr[i] as u16;
                    let num = 1 << n;
                    if check & num != 0 {
                        return Some((i % 4, i / 4));
                    }
                    check |= num;
                }
            }
            check = 0;
            // Check row
            for x in 0..16 {
                let n = self[y][x];
                let num = 1 << n;
                if check & num != 0 {
                    return Some((x, y));
                }
                check |= num;
            }
            // Check col
            check = 0;
            let x = y;
            for y in 0..16 {
                let n = self[y][x];
                let num = 1 << n;
                if check & num != 0 {
                    return Some((x, y));
                }
                check |= num;
            }
        }
        // Check last col
        check = 0;
        for y in 0..16 {
            let n = self[y][8];
            let num = 1 << n;
            if check & num != 0 {
                return Some((15, y));
            }
            check |= num;
        }
        None
    }

    pub fn get_row_for(&self, (_, y): Pos, arr: &mut [u8; 16]) {
        for x in 0..16 {
            arr[x] = self[y][x];
        }
    }

    pub fn get_col_for(&self, (x, _): Pos, arr: &mut [u8; 16]) {
        for y in 0..16 {
            arr[y] = self[y][x];
        }
    }

    pub fn get_box_for(&self, (x, y): Pos, arr: &mut [u8; 16]) {
        let (x, y) = (x / 4 * 4, y / 4 * 4);
        let mut i = 0;
        for y in y..y + 4 {
            for x in x..x + 4 {
                arr[i] = self[y][x];
                i += 1;
            }
        }
    }

    fn make_row_wise(&mut self) {
        match self.1 {
            GridLayout::Row => return,
            GridLayout::Col => self.reflect_y_x(),
            GridLayout::Box => self.make_box_from_row(),
        }
        self.1 = GridLayout::Row;
    }

    fn make_col_wise(&mut self) {
        match self.1 {
            GridLayout::Col => return,
            GridLayout::Row => self.reflect_y_x(),
            GridLayout::Box => self.make_box_from_col(),
        }
        self.1 = GridLayout::Col;
    }

    fn make_box_wise(&mut self) {
        match self.1 {
            GridLayout::Box => return,
            GridLayout::Row => self.make_box_from_row(),
            GridLayout::Col => self.make_box_from_col(),
        }
        self.1 = GridLayout::Box;
    }

    fn make_box_from_row(&mut self) {
        for i in 0..4 {
            let (mut y1, mut y2) = (i * 4 + 1, i * 4);
            for x in 0..4 {
                self.swap_pos((x, y1), (x + 4, y2));
            }
            y1 += 1;
            for x in 0..4 {
                self.swap_pos((x, y1), (x + 8, y2));
            }
            y1 += 1;
            for x in 0..4 {
                self.swap_pos((x, y1), (x + 12, y2));
            }

            y2 += 1;
            for x in 4..8 {
                self.swap_pos((x, y1), (x + 8, y2));
            }
            y1 -= 1;
            for x in 4..8 {
                self.swap_pos((x, y1), (x + 4, y2));
            }

            y1 += 1;
            y2 += 1;
            for x in 8..12 {
                self.swap_pos((x, y1), (x + 4, y2));
            }
        }
    }

    fn make_box_from_col(&mut self) {
        for i in 0..4 {
            let (mut x1, mut x2) = (i * 4 + 1, i * 4);
            for y in 0..4 {
                self.swap_pos((x1, y), (x2, y + 4));
            }
            x1 += 1;
            for y in 0..4 {
                self.swap_pos((x1, y), (x2, y + 8));
            }
            x1 += 1;
            for y in 0..4 {
                self.swap_pos((x1, y), (x2, y + 12));
            }

            x2 += 1;
            for y in 4..8 {
                self.swap_pos((x1, y), (x2, y + 8));
            }
            x1 -= 1;
            for y in 4..8 {
                self.swap_pos((x1, y), (x2, y + 4));
            }

            x1 += 1;
            x2 += 1;
            for y in 8..12 {
                self.swap_pos((x1, y), (x2, y + 4));
            }
        }
    }

    // Reflect on y = x
    #[inline]
    fn reflect_y_x(&mut self) {
        for y in 0..16 {
            for x in y + 1..16 {
                self.swap_pos((x, y), (y, x));
            }
        }
    }

    // Reflect on y = -x (i.e., transpose)
    #[inline]
    fn reflect_y_neg_x(&mut self) {
        for y in 0..16 {
            // (0..16 - y - 1)
            for x in (0..15 - y).rev() {
                self.swap_pos((x, y), (15 - y, 15 - x));
            }
        }
    }

    // Source: https://www.geeksforgeeks.org/rotate-a-matrix-by-90-degree-in-clockwise-direction-without-using-any-extra-space/
    fn rotate_90(&mut self) {
        // 0..16 / 2
        for y in 0..8 {
            // (0..16 - y - 1)
            for x in y..15 - y {
                let temp = self[y][x];
                // All 15s are just 16 - 1 condensed
                self[y][x] = self[15 - x][y];
                self[15 - x][y] = self[15 - y][15 - x];
                self[15 - y][15 - x] = self[x][15 - y];
                self[x][15 - y] = temp;
            }
        }
    }

    fn rotate_180(&mut self) {
        for y in 0..8 {
            for x in 0..16 {
                self.swap_pos((x, y), (15 - x, 15 - y));
            }
        }
    }

    // Source: https://www.enjoyalgorithms.com/blog/rotate-a-matrix-by-90-degrees-in-an-anticlockwise-direction
    fn rotate_270(&mut self) {
        for y in 0..8 {
            // (0..16 - y - 1)
            for x in y..15 - y {
                let temp = self[y][x];
                // All 15s are just 16 - 1 condensed
                self[y][x] = self[x][15 - y];
                self[x][15 - y] = self[15 - y][15 - x];
                self[15 - y][15 - x] = self[15 - x][y];
                self[15 - x][y] = temp;
            }
        }
    }

    fn reflect_x(&mut self) {
        for y in 0..8 {
            self.0.swap(y, 15 - y);
        }
    }

    fn reflect_y(&mut self) {
        for y in 0..16 {
            for x in 0..8 {
                self[y].swap(x, 15 - x);
            }
        }
    }

    #[inline]
    fn swap_pos(&mut self, (x1, y1): Pos, (x2, y2): Pos) {
        // Make sure they aren't the same position
        if x1 != x2 || y1 != y2 {
            unsafe {
                // Rust doesn't allow the second mutable borrow even though they
                // individual elements referenced won't share any memory at all.
                let r1 = &mut self.0[y1][x1] as *mut u8;
                let r2 = &mut self.0[y2][x2] as *mut u8;
                swap(&mut *r1, &mut *r2);
            }
        }
    }
}

impl Index<Pos> for Grid4x4 {
    type Output = u8;
    fn index(&self, (x, y): Pos) -> &Self::Output {
        &self.0[y][x]
    }
}

impl IndexMut<Pos> for Grid4x4 {
    fn index_mut(&mut self, (x, y): Pos) -> &mut Self::Output {
        &mut self.0[y][x]
    }
}

impl Index<usize> for Grid4x4 {
    type Output = [u8; 16];
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Grid4x4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl fmt::Display for Grid4x4 {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..16 {
            for x in 0..16 {
                let c = match self.0[y][x] {
                    0 => '_',
                    b @ 1..=9 => (b + b'0') as char,
                    b => (b'A' + b - 10) as char,
                };
                match x {
                    3 | 7 | 11 => write!(w, "{} | ", c)?,
                    15 => write!(w, "{}", c)?,
                    _ => write!(w, "{} ", c)?,
                }
            }
            if y % 4 == 3 && y != 15 {
                write!(w, "\n-------------------------------------\n")?;
            } else {
                write!(w, "\n")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub pos: Pos,
    pub num: u8,
    pub prev_num: u8,
}

impl Move {
    pub fn new(pos: Pos, num: u8, prev_num: u8) -> Self {
        Self { pos, num, prev_num }
    }
}

#[derive(Clone, Default)]
pub struct History {
    moves: Vec<Move>,
    // Keeps track of where we are in history. pos = 0 means there is no history
    pos: usize,
}

impl History {
    pub fn new() -> Self {
        Default::default()
    }

    // Adds a move to the history
    pub fn push(&mut self, mv: Move) {
        self.moves.push(mv);
        self.pos += 1;
    }

    // Returns true if there is another undo possible.
    pub fn can_undo(&self) -> bool {
        self.pos != 0
    }

    // Gets the move from the current history and decrements the current history position, if
    // possible
    pub fn undo(&mut self) -> Option<Move> {
        if !self.can_undo() {
            // No history left to undo
            return None;
        }
        self.pos -= 1;
        Some(self.moves[self.pos])
    }

    // Returns true if there is another redo possible.
    pub fn can_redo(&self) -> bool {
        self.pos != self.moves.len()
    }

    // Gets the next move from the current history and increments the current history position, if
    // possible
    pub fn redo(&mut self) -> Option<Move> {
        if !self.can_redo() {
            // No history left to redo
            return None;
        }
        let mv = Some(self.moves[self.pos]);
        self.pos += 1;
        mv
    }

    // Replaces the current position with the new move, clearing the history that follows.
    // If there is no history, it pushes it.
    pub fn replace(&mut self, mv: Move) {
        if self.pos == 0 {
            self.push(mv);
            return;
        }
        self.moves[self.pos - 1] = mv;
        self.moves.drain(self.pos..);
    }
}

#[cfg(test)]
mod tests {
    use crate::Grid3x3;

    #[test]
    fn row_col_box_3x3() {
        let grid = Grid3x3::new();
        let mut got = [0u8; 9];

        grid.get_row_for((0, 0), &mut got);
        assert_eq!(got, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
        grid.get_col_for((0, 0), &mut got);
        assert_eq!(got, [1, 4, 7, 2, 5, 8, 3, 6, 9]);
        grid.get_box_for((0, 0), &mut got);
        assert_eq!(got, [1, 2, 3, 4, 5, 6, 7, 8, 9]);

        grid.get_row_for((2, 1), &mut got);
        assert_eq!(got, [4, 5, 6, 7, 8, 9, 1, 2, 3]);
        grid.get_col_for((2, 1), &mut got);
        assert_eq!(got, [3, 6, 9, 4, 7, 1, 5, 8, 2]);
        grid.get_box_for((2, 1), &mut got);
        assert_eq!(got, [1, 2, 3, 4, 5, 6, 7, 8, 9]);

        grid.get_row_for((4, 4), &mut got);
        assert_eq!(got, [5, 6, 7, 8, 9, 1, 2, 3, 4]);
        grid.get_col_for((4, 4), &mut got);
        assert_eq!(got, [5, 8, 2, 6, 9, 3, 7, 1, 4]);
        grid.get_box_for((4, 4), &mut got);
        assert_eq!(got, [5, 6, 7, 8, 9, 1, 2, 3, 4]);
    }

    #[test]
    fn pos_is_valid_3x3() {
        let mut grid = Grid3x3::empty();
        grid[0][0] = 1;
        grid[4][1] = 1;
        grid[1][4] = 1;
        assert!(!grid.pos_is_valid((1, 1), 1));
        assert!(!grid.pos_is_valid((2, 2), 1));
        assert!(!grid.pos_is_valid((8, 1), 1));
        assert!(!grid.pos_is_valid((1, 8), 1));

        assert!(grid.pos_is_valid((1, 1), 2));
        assert!(grid.pos_is_valid((2, 2), 2));
        assert!(grid.pos_is_valid((8, 1), 2));
        assert!(grid.pos_is_valid((1, 8), 2));
    }
}
