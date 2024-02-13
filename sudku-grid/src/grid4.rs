use crate::{GridLayout, History, Move, MultiHistory, Pos};
use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
    Rng,
};
use std::fmt;
use std::mem::swap;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Num4x4(u32);

impl Num4x4 {
    const NOTE_BIT: u32 = 1 << 31;
    const GIVEN_BIT: u32 = 1 << 30;

    pub const fn new(num: u8) -> Self {
        Self(num as _)
    }

    pub fn new_note(num: u8) -> Self {
        if num == 0 {
            Self(0)
        } else {
            Self(Self::note_for_num(num))
        }
    }

    pub fn num(self) -> Option<u8> {
        (!self.is_note()).then_some(self.0 as u8)
    }

    pub fn num_or_zero(self) -> u8 {
        self.num().unwrap_or(0)
    }

    pub fn is_note(self) -> bool {
        self.0 & Self::NOTE_BIT != 0
    }

    pub fn is_given(self) -> bool {
        self.0 & Self::GIVEN_BIT != 0
    }

    pub fn has_note(self, num: u8) -> Option<bool> {
        (num != 0 && self.is_note()).then_some(self.0 & (1 << (num - 1) as u32) != 0)
    }

    pub fn notes(self) -> Option<[bool; 16]> {
        let mut arr = [false; 16];
        for i in 1..=16 {
            arr[i - 1] = self.has_note(i as _)?;
        }
        Some(arr)
    }

    pub fn with_num(self, num: u8) -> Self {
        Self(num as _)
    }

    pub fn with_note(self, num: u8) -> Self {
        // TODO: check with is_note()?
        if self.0 <= 16 {
            return Self(Self::note_for_num(num));
        }
        Self(self.0 | Self::note_for_num(num))
    }

    pub fn with_notes(mut self, nums: [bool; 16]) -> Self {
        for n in nums
            .into_iter()
            .enumerate()
            .filter_map(|(i, b)| b.then_some(i + 1))
        {
            self = self.with_note(n as _);
        }
        self
    }

    pub fn with_toggle_note(self, num: u8) -> Self {
        // TODO: check with is_note()?
        if self.0 <= 16 {
            return Self(Self::note_for_num(num));
        }
        Self(Self::NOTE_BIT | (self.0 ^ Self::note_for_num(num)))
    }

    pub fn set_num(&mut self, num: u8) {
        self.0 = num as _;
    }

    pub fn set_note(&mut self, num: u8) {
        // TODO: check with is_note()?
        if self.0 <= 16 {
            self.0 = Self::note_for_num(num);
            return;
        }
        self.0 |= Self::note_for_num(num);
    }

    // Returns whether the note is set by the function (true) or unset (false).
    pub fn set_toggle_note(&mut self, num: u8) -> bool {
        let note = Self::note_for_num(num);
        // TODO: check with is_note()?
        if self.0 <= 16 {
            self.0 = note;
            return true;
        }
        let not_set = self.0 & note == 0;
        *self = self.with_toggle_note(num);
        not_set
    }

    pub fn set_given(&mut self) {
        self.0 = Self::GIVEN_BIT | self.num_or_zero() as u32;
    }

    #[inline(always)]
    fn note_for_num(num: u8) -> u32 {
        assert_ne!(num, 0);
        Self::NOTE_BIT | (1 << num as u32 - 1)
    }
}

pub type Move4x4 = Move<Num4x4>;
pub type History4x4 = History<Num4x4>;
pub type MultiHistory4x4 = MultiHistory<Num4x4>;

pub type Nums4x4 = [[Num4x4; 16]; 16];

pub const BASE_4X4: Nums4x4 = [
    [
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
    ],
    [
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
    ],
    [
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
    ],
    [
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
    ],
    [
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
    ],
    [
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
    ],
    [
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
    ],
    [
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
    ],
    [
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
    ],
    [
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
    ],
    [
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
    ],
    [
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
    ],
    [
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
    ],
    [
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
    ],
    [
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
    ],
    [
        Num4x4(16),
        Num4x4(1),
        Num4x4(2),
        Num4x4(3),
        Num4x4(4),
        Num4x4(5),
        Num4x4(6),
        Num4x4(7),
        Num4x4(8),
        Num4x4(9),
        Num4x4(10),
        Num4x4(11),
        Num4x4(12),
        Num4x4(13),
        Num4x4(14),
        Num4x4(15),
    ],
];

pub const EMPTY_4X4: Nums4x4 = [[Num4x4(0); 16]; 16];

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Grid4x4([[Num4x4; 16]; 16], GridLayout);

impl Grid4x4 {
    const ENCODED_LEN: usize = 16 * 16 * 4;

    pub const EMPTY: Self = Self(EMPTY_4X4, GridLayout::Row);

    pub const fn new() -> Self {
        Self(BASE_4X4, GridLayout::Row)
    }

    pub fn empty() -> Self {
        Self([[Num4x4(0); 16]; 16], GridLayout::Row)
    }

    pub fn randomized() -> Self {
        let mut grid = Self::new();
        grid.randomize();
        grid
    }

    pub fn generate() -> Self {
        let mut grid = Self::empty();
        let mut rng = rand::thread_rng();
        let infos = (0..256)
            .map(|i| {
                let pos = (i % 16, i / 16);
                let mut nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
                nums.shuffle(&mut rng);
                (pos, nums)
            })
            .collect::<Vec<_>>();
        // TODO?
        assert!(grid.gen_helper(&infos), "failed to generate board");
        grid
    }

    /// returns true if all good
    fn gen_helper(&mut self, infos: &[(Pos, [u8; 16])]) -> bool {
        let Some((pos, rand_nums)) = infos.get(0).copied() else {
            return true;
        };
        for num in rand_nums {
            if self.try_place(pos, Num4x4(num as _)) {
                if self.gen_helper(&infos[1..]) {
                    return true;
                }
            }
        }
        self[pos] = Num4x4(0);
        false
    }

    /// Returns false if the slice passed is too small
    pub fn from_encoded(encoded: impl AsRef<[u8]>) -> Option<Self> {
        let mut grid = Self::empty();
        if grid.decode_from(encoded) {
            Some(grid)
        } else {
            None
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(Self::ENCODED_LEN);
        for row in &self.0 {
            for num in row {
                let bytes = num.0.to_be_bytes();
                res.push(bytes[0]);
                res.push(bytes[1]);
                res.push(bytes[2]);
                res.push(bytes[3]);
            }
        }
        res
    }

    /// Returns false if the slice passed is too small
    pub fn decode_from(&mut self, encoded: impl AsRef<[u8]>) -> bool {
        let encoded = encoded.as_ref();
        if encoded.len() < Self::ENCODED_LEN {
            return false;
        }
        for (i, s) in encoded[..Self::ENCODED_LEN].chunks(4).enumerate() {
            self[i / 16][i % 16] = Num4x4(u32::from_be_bytes([s[0], s[1], s[2], s[3]]));
        }
        true
    }

    pub fn randomize(&mut self) {
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
                }
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

        self.make_box_wise();
        /*
        for num1 in 1..=15 {
            for num2 in num1 + 1..=16 {
                for bn in 0..16 {
                    let bx = &mut self[bn];
                    let (mut i1, mut i2) = (usize::MAX, usize::MAX);
                    for (i, n) in bx.into_iter().enumerate() {
                        if n.num_or_zero() == num1 {
                            i1 = i;
                            if i2 != usize::MAX {
                                break;
                            }
                        } else if n.num_or_zero() == num2 {
                            i2 = i;
                            if i1 != usize::MAX {
                                break;
                            }
                        }
                    }
                    if i1 == usize::MAX || i2 == usize::MAX {
                        // TODO
                        return;
                    }
                    bx.swap(i1, i2);
                }
            }
        }
        */
        for _ in 0..num_transforms {
            let num1 = of_sixteen.sample(&mut rng);
            let num2 = loop {
                let n = of_sixteen.sample(&mut rng);
                if n != num1 {
                    break n;
                }
            };
            let (num1, num2) = (num1 as u8 + 1, num2 as u8 + 1);
            for bn in 0..16 {
                let bx = &mut self[bn];
                let (mut i1, mut i2) = (usize::MAX, usize::MAX);
                for (i, n) in bx.into_iter().enumerate() {
                    if n.num_or_zero() == num1 {
                        i1 = i;
                        if i2 != usize::MAX {
                            break;
                        }
                    } else if n.num_or_zero() == num2 {
                        i2 = i;
                        if i1 != usize::MAX {
                            break;
                        }
                    }
                }
                if i1 == usize::MAX || i2 == usize::MAX {
                    // TODO
                    return;
                }
                bx.swap(i1, i2);
            }
        }
        self.make_row_wise();

        // Sanity check
        // TODO: Possibly panic
        if let Some(_pos) = self.is_valid() {
            self.randomize();
        }
    }

    // n is the number to remove
    // TODO: return error if too many are attempted to be removed
    pub fn remove_nums(&mut self, n_remove: usize) -> usize {
        assert!(n_remove < 256, "cannot remove more symbols than exists");
        let mut indexes = (0..256usize).collect::<Vec<usize>>();
        let mut rng = rand::thread_rng();
        indexes.shuffle(&mut rng);

        // At least n^2 - 1 distict symbols must be kept when removing symbols to have a unique
        // solution (having n^2 - 1 does not mean the solution is unique, though).
        // Source: https://pi.math.cornell.edu/~mec/Summer2009/Mahmood/More.html

        // Keeps track of whether the one number that can be fully removed has been
        let mut num_gone = false;
        // Allocate 10 instad of 9 so that the nums from the squares can be converted from their
        // values to a usize without having to subtract 1 to get the index and worry about zeros
        let mut nums = [16u8; 17];
        let mut num_removed = 0;
        for i in indexes {
            if num_removed == n_remove {
                break;
            }
            let (x, y) = (i / 16, i % 16);
            let num = self[y][x].num_or_zero() as usize;
            if num != 0 && (!num_gone || nums[num] != 1) {
                self[y][x] = Num4x4(0);
                nums[num] -= 1;
                num_removed += 1;
                if nums[num] == 0 {
                    num_gone = true;
                }
            }
        }
        num_removed
    }

    // Sets all non-zero numbers as given
    pub fn set_given(&mut self) {
        for y in 0..16 {
            for x in 0..16 {
                if self[y][x].num_or_zero() != 0 {
                    self[y][x].set_given();
                }
            }
        }
    }

    // Returns true if the number can be placed in the square
    pub fn pos_is_valid(&self, pos: Pos, n: u8) -> bool {
        if n == 0 {
            return true;
        }

        let mut arr = [Num4x4(0); 16];
        self.get_row_for(pos, &mut arr);
        if arr.iter().any(|num| num.num_or_zero() == n) {
            return false;
        }
        self.get_col_for(pos, &mut arr);
        if arr.iter().any(|num| num.num_or_zero() == n) {
            return false;
        }
        self.get_box_for(pos, &mut arr);
        !arr.iter().any(|num| num.num_or_zero() == n)
    }

    /// Returns true if placed
    pub fn try_place(&mut self, pos: Pos, num: Num4x4) -> bool {
        if self.pos_is_valid(pos, num.num_or_zero()) {
            self[pos] = num;
            true
        } else {
            false
        }
    }

    // Returns None if valid, otherwise, returns the position of the first bad square encountered.
    // TODO: Try to optimize better
    pub fn is_valid(&self) -> Option<Pos> {
        // TODO: Do better
        for x in 0..16 {
            for y in 0..16 {
                if self[(x, y)].num_or_zero() == 0 {
                    return Some((x, y));
                }
            }
        }

        let mut arr = [Num4x4(0); 16];
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
                    let n = arr[i].num_or_zero() as u16;
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
                let n = self[y][x].num_or_zero();
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
                let n = self[y][x].num_or_zero();
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
            let n = self[y][15].num_or_zero();
            let num = 1 << n;
            if check & num != 0 {
                return Some((15, y));
            }
            check |= num;
        }
        None
    }

    pub fn get_row_for(&self, (_, y): Pos, arr: &mut [Num4x4; 16]) {
        for x in 0..16 {
            arr[x] = self[y][x];
        }
    }

    pub fn get_col_for(&self, (x, _): Pos, arr: &mut [Num4x4; 16]) {
        for y in 0..16 {
            arr[y] = self[y][x];
        }
    }

    pub fn get_box_for(&self, (x, y): Pos, arr: &mut [Num4x4; 16]) {
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
                let r1 = &mut self.0[y1][x1] as *mut Num4x4;
                let r2 = &mut self.0[y2][x2] as *mut Num4x4;
                swap(&mut *r1, &mut *r2);
            }
        }
    }

    /// Returns true if a solution was found. Assumes the board is not already in an invalid state.
    pub fn solve(&mut self) -> bool {
        for y in 0..16 {
            for x in 0..16 {
                if self[(x, y)].num_or_zero() == 0 {
                    return self.solve_helper((x, y));
                }
            }
        }
        true
    }

    fn solve_helper(&mut self, pos: Pos) -> bool {
        let old = self[pos];
        for n in 1..=16 {
            if self.try_place(pos, Num4x4(n as _)) {
                let mut new_pos = (0, 0);
                'outer: for y in pos.1..16 {
                    for x in pos.0..16 {
                        if self[(x, y)].num_or_zero() == 0 {
                            new_pos = (x, y);
                            break 'outer;
                        }
                    }
                }
                if new_pos == (0, 0) {
                    return true;
                }
                if self.solve_helper(new_pos) {
                    return true;
                }
            }
        }
        self[pos] = old;
        false
    }
}

impl Index<Pos> for Grid4x4 {
    type Output = Num4x4;
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
    type Output = [Num4x4; 16];
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
                let c = match self.0[y][x].num_or_zero() {
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

// TODO: Test
