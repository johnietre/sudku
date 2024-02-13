pub use rand;

pub mod grid3;
pub use grid3::*;

pub mod grid4;
pub use grid4::*;

pub mod history;
pub use history::*;

//pub mod multi_history;

pub type Pos = (usize, usize);

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum GridLayout {
    #[default]
    Row,
    Col,
    Box,
}
