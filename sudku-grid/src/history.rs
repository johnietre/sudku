use crate::Pos;
use std::fmt;
use std::ptr::NonNull;

#[derive(Clone)]
pub struct Move<N: Clone> {
    pub old: N,
    pub new: N,
    pub pos: Pos,
}

impl<N: Clone> Move<N> {
    pub fn new(old: N, new: N, pos: Pos) -> Self {
        Self { old, new, pos }
    }
}

impl<N: Clone + PartialEq> PartialEq for Move<N> {
    fn eq(&self, other: &Self) -> bool {
        self.old == other.old && self.new == other.new && self.pos == other.pos
    }
}

impl<N: Clone + Eq> Eq for Move<N> {}

impl<N: Copy> Copy for Move<N> {}

impl<N: Clone + fmt::Debug> fmt::Debug for Move<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Move")
            .field("old", &self.old)
            .field("new", &self.new)
            .field("pos", &self.pos)
            .finish()
    }
}

type ONode<N> = Option<NonNull<Node<N>>>;

struct Node<N: Clone> {
    mv: Move<N>,
    prev: ONode<N>,
    next: ONode<N>,
}

impl<N: Clone> Node<N> {
    fn new(mv: Move<N>, prev: ONode<N>, next: ONode<N>) -> Self {
        Self { mv, prev, next }
    }

    fn new_onode(mv: Move<N>, prev: ONode<N>, next: ONode<N>) -> ONode<N> {
        Self::new(mv, prev, next).to_onode()
    }

    fn to_onode(self) -> ONode<N> {
        NonNull::new(Box::into_raw(Box::new(self)))
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum Where {
    Front,
    Middle,
    #[default]
    Back,
}

#[derive(Default)]
pub struct History<N: Clone> {
    curr: ONode<N>,
    loc: Where,
}

impl<N: Clone> History<N> {
    pub const fn new() -> Self {
        Self {
            curr: None,
            loc: Where::Back,
        }
    }

    /// Adds a new move to the history, discarding the remaining history if not currently on the
    /// latest move (i.e., if undo has been called without the equivalent number of calls to redo).
    pub fn update(&mut self, mv: Move<N>) {
        unsafe {
            match self.loc {
                Where::Back => {
                    let mv = Node::new_onode(mv, self.curr, None);
                    if let Some(mut curr) = self.curr {
                        curr.as_mut().next = mv;
                    }
                    self.curr = mv;
                }
                Where::Front => {
                    *self = Self {
                        curr: Node::new_onode(mv, None, None),
                        loc: Where::Back,
                    }
                }
                Where::Middle => {
                    let mv = Node::new_onode(mv, self.curr, None);
                    if let Some(mut curr) = self.curr {
                        if let Some(mut next) = curr.as_ref().next {
                            next.as_mut().prev = None;
                            // Free remaining history
                            History {
                                curr: Some(next),
                                loc: Where::Front,
                            };
                        }
                        curr.as_mut().next = mv;
                    }
                    self.curr = mv;
                }
            }
        }
        self.loc = Where::Back;
    }
    pub fn can_undo(&self) -> bool {
        self.loc != Where::Front && self.curr.is_some()
    }

    pub fn can_redo(&self) -> bool {
        self.loc != Where::Back && self.curr.is_some()
    }

    pub fn undo<'a>(&'a mut self) -> Option<&Move<N>> {
        if !self.can_undo() {
            return None;
        }
        unsafe {
            let curr = self.curr?.as_ref();
            //let mv = curr.mv;
            if curr.prev.is_some() {
                self.curr = curr.prev;
                self.loc = Where::Middle;
            } else {
                self.loc = Where::Front;
            }
            Some(&curr.mv)
        }
    }

    pub fn redo<'a>(&'a mut self) -> Option<&'a Move<N>> {
        if !self.can_redo() {
            return None;
        }
        unsafe {
            if self.loc == Where::Front {
                let node = self.curr?.as_ref();
                self.loc = if node.next.is_some() {
                    Where::Middle
                } else {
                    Where::Back
                };
                Some(&node.mv)
            } else {
                let next = self.curr?.as_ref().next?;
                self.curr = Some(next);
                self.loc = if next.as_ref().next.is_some() {
                    Where::Middle
                } else {
                    Where::Back
                };
                Some(&next.as_ref().mv)
            }
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }
}

impl<N: Clone + PartialEq> History<N> {
    pub fn same_as(&self, other: &Self) -> bool {
        unsafe {
            let Some(mut sf) = self.curr else {
                return other.curr.is_none();
            };
            while let Some(p) = sf.as_ref().prev {
                sf = p;
            }
            let Some(mut of) = other.curr else {
                return false;
            };
            while let Some(p) = of.as_ref().prev {
                of = p;
            }
            let (mut sf, mut of) = (Some(sf), Some(of));
            while let Some((s, o)) = sf.zip(of).map(|(s, o)| (s.as_ref(), o.as_ref())) {
                if s.mv != o.mv {
                    return false;
                }
                sf = s.next;
                of = o.next;
            }
            sf.is_none() && of.is_none()
        }
    }
}

impl<N: Clone + PartialEq> PartialEq for History<N> {
    fn eq(&self, other: &Self) -> bool {
        if self.loc != other.loc {
            return false;
        }
        unsafe {
            let Some(sf) = self.curr.map(|c| c.as_ref()) else {
                return other.curr.is_none();
            };
            let Some(of) = other.curr.map(|c| c.as_ref()) else {
                return false;
            };
            if sf.mv != of.mv {
                return false;
            }

            let (mut sp, mut op) = (sf.prev, of.prev);
            while let Some((s, o)) = sp.zip(op).map(|(s, o)| (s.as_ref(), o.as_ref())) {
                if s.mv != o.mv {
                    return false;
                }
                sp = s.prev;
                op = o.prev;
            }
            if sp.is_some() || op.is_some() {
                return false;
            }

            let (mut sn, mut on) = (sf.next, of.next);
            while let Some((s, o)) = sn.zip(on).map(|(s, o)| (s.as_ref(), o.as_ref())) {
                if s.mv != o.mv {
                    return false;
                }
                sn = s.next;
                on = o.next;
            }
            sn.is_none() && on.is_none()
        }
    }
}

impl<N: Clone + Eq> Eq for History<N> {}

impl<N: Clone> Clone for History<N> {
    fn clone(&self) -> Self {
        unsafe {
            let Some(curr) = self.curr.map(|c| c.as_ref()) else {
                return Self::new();
            };
            let new_curr = Node::new_onode(curr.mv.clone(), None, None);
            let new_curr_ref = new_curr.unwrap().as_mut();

            if let Some(prev) = curr.prev.map(|p| p.as_ref()) {
                new_curr_ref.prev = Node::new_onode(prev.mv.clone(), None, new_curr);
                let mut new_prev = new_curr_ref.prev.unwrap();
                let mut prev = prev.prev;
                while let Some(p) = prev.map(|p| p.as_ref()) {
                    new_prev.as_mut().prev = Node::new_onode(p.mv.clone(), None, Some(new_prev));
                    new_prev = new_prev.as_ref().prev.unwrap();
                    prev = p.prev;
                }
            }
            if let Some(next) = curr.next.map(|n| n.as_ref()) {
                new_curr_ref.next = Node::new_onode(next.mv.clone(), None, new_curr);
                let mut new_next = new_curr_ref.next.unwrap();
                let mut next = next.next;
                while let Some(n) = next.map(|n| n.as_ref()) {
                    new_next.as_mut().next = Node::new_onode(n.mv.clone(), None, Some(new_next));
                    new_next = new_next.as_ref().next.unwrap();
                    next = n.next;
                }
            }
            Self {
                curr: new_curr,
                loc: self.loc,
            }
        }
    }
}

impl<N: Clone + fmt::Debug> fmt::Debug for History<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let Some(curr) = self.curr.map(|c| c.as_ref()) else {
                return write!(f, "[|]");
            };

            let mut mvs = Vec::new();
            let mut prev = curr.prev;
            while let Some(p) = prev.map(|p| p.as_ref()) {
                mvs.push(&p.mv);
                prev = p.prev;
            }
            if self.loc == Where::Front {
                write!(f, "[|")?;
            } else {
                write!(f, "[")?;
            }
            for mv in mvs.into_iter().rev() {
                write!(f, "{mv:?}, ")?;
            }

            if self.loc == Where::Middle {
                write!(f, "|{:?}|", curr.mv)?;
            } else {
                write!(f, "{:?}", curr.mv)?;
            }

            let mut next = curr.next;
            while let Some(n) = next.map(|n| n.as_ref()) {
                write!(f, ", {:?}", n.mv)?;
                next = n.next;
            }
            if self.loc == Where::Back {
                write!(f, "|]")
            } else {
                write!(f, "]")
            }
        }
    }
}

impl<N: Clone> Drop for History<N> {
    fn drop(&mut self) {
        unsafe {
            let Some(mut curr) = self.curr.take().map(|c| Box::from_raw(c.as_ptr())) else {
                return;
            };
            while let Some(prev) = curr.prev {
                let prev = Box::from_raw(prev.as_ptr());
                curr.prev = prev.prev;
            }
            while let Some(next) = curr.next {
                let next = Box::from_raw(next.as_ptr());
                curr.next = next.next;
            }
        }
    }
}

type OMultiNode<N> = Option<NonNull<MultiNode<N>>>;

struct MultiNode<N: Clone> {
    mvs: Vec<Move<N>>,
    prev: OMultiNode<N>,
    next: OMultiNode<N>,
}

impl<N: Clone> MultiNode<N> {
    fn new(mvs: Vec<Move<N>>, prev: OMultiNode<N>, next: OMultiNode<N>) -> Self {
        Self { mvs, prev, next }
    }

    fn new_onode(mvs: Vec<Move<N>>, prev: OMultiNode<N>, next: OMultiNode<N>) -> OMultiNode<N> {
        Self::new(mvs, prev, next).to_onode()
    }

    fn to_onode(self) -> OMultiNode<N> {
        NonNull::new(Box::into_raw(Box::new(self)))
    }
}

#[derive(Default)]
pub struct MultiHistory<N: Clone> {
    curr: OMultiNode<N>,
    loc: Where,
}

impl<N: Clone> MultiHistory<N> {
    pub const fn new() -> Self {
        Self {
            curr: None,
            loc: Where::Back,
        }
    }

    /// Adds a new move set to the history, discarding the remaining history if not currently on the
    /// latest move (i.e., if undo has been called without the equivalent number of calls to redo).
    pub fn update(&mut self, mvs: Vec<Move<N>>) {
        unsafe {
            match self.loc {
                Where::Back => {
                    let node = MultiNode::new_onode(mvs, self.curr, None);
                    if let Some(mut curr) = self.curr {
                        curr.as_mut().next = node;
                    }
                    self.curr = node;
                }
                Where::Front => {
                    *self = Self {
                        curr: MultiNode::new_onode(mvs, None, None),
                        loc: Where::Back,
                    }
                }
                Where::Middle => {
                    let node = MultiNode::new_onode(mvs, self.curr, None);
                    if let Some(mut curr) = self.curr {
                        if let Some(mut next) = curr.as_ref().next {
                            next.as_mut().prev = None;
                            // Free remaining history
                            MultiHistory {
                                curr: Some(next),
                                loc: Where::Front,
                            };
                        }
                        curr.as_mut().next = node;
                    }
                    self.curr = node;
                }
            }
        }
        self.loc = Where::Back;
    }
    pub fn can_undo(&self) -> bool {
        self.loc != Where::Front && self.curr.is_some()
    }

    pub fn can_redo(&self) -> bool {
        self.loc != Where::Back && self.curr.is_some()
    }

    pub fn undo<'a>(&'a mut self) -> Option<&'a Vec<Move<N>>> {
        if !self.can_undo() {
            return None;
        }
        unsafe {
            let curr = self.curr?.as_ref();
            //let mv = curr.mv;
            if curr.prev.is_some() {
                self.curr = curr.prev;
                self.loc = Where::Middle;
            } else {
                self.loc = Where::Front;
            }
            Some(&curr.mvs)
        }
    }

    pub fn redo<'a>(&'a mut self) -> Option<&'a Vec<Move<N>>> {
        if !self.can_redo() {
            return None;
        }
        unsafe {
            if self.loc == Where::Front {
                let node = self.curr?.as_ref();
                self.loc = if node.next.is_some() {
                    Where::Middle
                } else {
                    Where::Back
                };
                Some(&node.mvs)
            } else {
                let next = self.curr?.as_ref().next?;
                self.curr = Some(next);
                self.loc = if next.as_ref().next.is_some() {
                    Where::Middle
                } else {
                    Where::Back
                };
                Some(&next.as_ref().mvs)
            }
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }
}

impl<N: Clone + PartialEq> MultiHistory<N> {
    pub fn same_as(&self, other: &Self) -> bool {
        unsafe {
            let Some(mut sf) = self.curr else {
                return other.curr.is_none();
            };
            while let Some(p) = sf.as_ref().prev {
                sf = p;
            }
            let Some(mut of) = other.curr else {
                return false;
            };
            while let Some(p) = of.as_ref().prev {
                of = p;
            }
            let (mut sf, mut of) = (Some(sf), Some(of));
            while let Some((s, o)) = sf.zip(of).map(|(s, o)| (s.as_ref(), o.as_ref())) {
                if s.mvs != o.mvs {
                    return false;
                }
                sf = s.next;
                of = o.next;
            }
            sf.is_none() && of.is_none()
        }
    }
}

impl<N: Clone + PartialEq> PartialEq for MultiHistory<N> {
    fn eq(&self, other: &Self) -> bool {
        if self.loc != other.loc {
            return false;
        }
        unsafe {
            let Some(sf) = self.curr.map(|c| c.as_ref()) else {
                return other.curr.is_none();
            };
            let Some(of) = other.curr.map(|c| c.as_ref()) else {
                return false;
            };
            if sf.mvs != of.mvs {
                return false;
            }

            let (mut sp, mut op) = (sf.prev, of.prev);
            while let Some((s, o)) = sp.zip(op).map(|(s, o)| (s.as_ref(), o.as_ref())) {
                if s.mvs != o.mvs {
                    return false;
                }
                sp = s.prev;
                op = o.prev;
            }
            if sp.is_some() || op.is_some() {
                return false;
            }

            let (mut sn, mut on) = (sf.next, of.next);
            while let Some((s, o)) = sn.zip(on).map(|(s, o)| (s.as_ref(), o.as_ref())) {
                if s.mvs != o.mvs {
                    return false;
                }
                sn = s.next;
                on = o.next;
            }
            sn.is_none() && on.is_none()
        }
    }
}

impl<N: Clone + Eq> Eq for MultiHistory<N> {}

impl<N: Clone> Clone for MultiHistory<N> {
    fn clone(&self) -> Self {
        unsafe {
            let Some(curr) = self.curr.map(|c| c.as_ref()) else {
                return Self::new();
            };
            let new_curr = MultiNode::new_onode(curr.mvs.clone(), None, None);
            let new_curr_ref = new_curr.unwrap().as_mut();

            if let Some(prev) = curr.prev.map(|p| p.as_ref()) {
                new_curr_ref.prev = MultiNode::new_onode(prev.mvs.clone(), None, new_curr);
                let mut new_prev = new_curr_ref.prev.unwrap();
                let mut prev = prev.prev;
                while let Some(p) = prev.map(|p| p.as_ref()) {
                    new_prev.as_mut().prev =
                        MultiNode::new_onode(p.mvs.clone(), None, Some(new_prev));
                    new_prev = new_prev.as_ref().prev.unwrap();
                    prev = p.prev;
                }
            }
            if let Some(next) = curr.next.map(|n| n.as_ref()) {
                new_curr_ref.next = MultiNode::new_onode(next.mvs.clone(), None, new_curr);
                let mut new_next = new_curr_ref.next.unwrap();
                let mut next = next.next;
                while let Some(n) = next.map(|n| n.as_ref()) {
                    new_next.as_mut().next =
                        MultiNode::new_onode(n.mvs.clone(), None, Some(new_next));
                    new_next = new_next.as_ref().next.unwrap();
                    next = n.next;
                }
            }
            Self {
                curr: new_curr,
                loc: self.loc,
            }
        }
    }
}

impl<N: Clone + fmt::Debug> fmt::Debug for MultiHistory<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let Some(curr) = self.curr.map(|c| c.as_ref()) else {
                return write!(f, "[|]");
            };

            let mut mvs_vec = Vec::new();
            let mut prev = curr.prev;
            while let Some(p) = prev.map(|p| p.as_ref()) {
                mvs_vec.push(&p.mvs);
                prev = p.prev;
            }
            if self.loc == Where::Front {
                write!(f, "[|")?;
            } else {
                write!(f, "[")?;
            }
            for mvs in mvs_vec.into_iter().rev() {
                write!(
                    f,
                    "{{{}}}, ",
                    mvs.iter()
                        .map(|mv| format!("{mv:?}"))
                        .collect::<Vec<_>>()
                        .join(","),
                )?;
            }

            if self.loc == Where::Middle {
                write!(
                    f,
                    "|{:?}|",
                    curr.mvs
                        .iter()
                        .map(|mv| format!("{mv:?}"))
                        .collect::<Vec<_>>()
                        .join(","),
                )?;
            } else {
                write!(
                    f,
                    "{:?}",
                    curr.mvs
                        .iter()
                        .map(|mv| format!("{mv:?}"))
                        .collect::<Vec<_>>()
                        .join(","),
                )?;
            }

            let mut next = curr.next;
            while let Some(n) = next.map(|n| n.as_ref()) {
                write!(
                    f,
                    ", {{{}}}",
                    n.mvs
                        .iter()
                        .map(|mv| format!("{mv:?}"))
                        .collect::<Vec<_>>()
                        .join(","),
                )?;
                next = n.next;
            }
            if self.loc == Where::Back {
                write!(f, "|]")
            } else {
                write!(f, "]")
            }
        }
    }
}

impl<N: Clone> Drop for MultiHistory<N> {
    fn drop(&mut self) {
        unsafe {
            let Some(mut curr) = self.curr.take().map(|c| Box::from_raw(c.as_ptr())) else {
                return;
            };
            while let Some(prev) = curr.prev {
                let prev = Box::from_raw(prev.as_ptr());
                curr.prev = prev.prev;
            }
            while let Some(next) = curr.next {
                let next = Box::from_raw(next.as_ptr());
                curr.next = next.next;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type IMove = Move<i32>;
    type IHistory = History<i32>;

    #[test]
    fn update_undo_redo() {
        let moves = (0..10)
            .map(|i| Move::new(i, i, (i as _, i as _)))
            .collect::<Vec<_>>();
        let mut hist = new_history(moves.clone());
        assert_eq!(hist.loc, Where::Back, "bad loc");

        for mv in moves.iter().copied().rev() {
            assert_eq!(hist.undo().copied(), Some(mv), "bad undo");
        }
        assert_eq!(hist.loc, Where::Front, "bad loc");
        assert_eq!(hist.undo(), None, "bad undo");
        for mv in moves.iter().copied() {
            assert_eq!(hist.redo().copied(), Some(mv), "bad redo");
        }
        assert_eq!(hist.loc, Where::Back, "bad loc");
        assert_eq!(hist.redo(), None, "bad redo");
        assert_eq!(hist, new_history(moves.clone()));

        let mv = Move::new(0, 0, (0, 0));
        hist.undo();
        assert_eq!(hist.loc, Where::Middle, "bad loc");
        hist.update(mv);
        assert_eq!(hist.redo(), None, "bad redo");

        assert_eq!(hist.undo().copied(), Some(mv), "bad undo");
        assert_eq!(hist.redo().copied(), Some(mv), "bad redo");

        for _ in moves.iter() {
            hist.undo();
        }
        let mv = Move::new(0, 0, (0, 0));
        assert_eq!(hist.loc, Where::Front);
        hist.update(mv);
        assert_eq!(hist.redo(), None, "bad redo");

        assert_eq!(hist.undo().copied(), Some(mv), "bad undo");
        assert_eq!(hist.undo(), None, "bad undo");

        assert_eq!(hist.redo().copied(), Some(mv), "bad redo");
        assert_eq!(hist.redo(), None, "bad redo");
    }

    #[test]
    fn clone_same_eq() {
        let mut hist = new_history((0..10).map(|i| Move::new(i, i, (i as _, i as _))));
        let mut histc = hist.clone();
        assert_eq!(hist, histc, "bad clone or eq");
        assert!(hist.same_as(&histc), "bad same as");

        histc.undo();
        assert_ne!(hist, histc, "bad eq");
        assert!(hist.same_as(&histc), "bad same as");

        histc.redo();
        assert_eq!(hist, histc, "bad eq");
        assert!(hist.same_as(&histc), "bad same as");

        histc.undo();
        histc.update(Move::new(-1, -1, (0, 0)));
        assert_ne!(hist, histc, "bad eq");
        assert!(!hist.same_as(&histc), "bad same as");

        histc.undo();
        histc.update(Move::new(9, 9, (9, 9)));
        hist.undo();
        assert_ne!(hist, histc, "bad eq");
        assert!(hist.same_as(&histc), "bad same as");

        hist.redo();
        assert_eq!(hist, histc, "bad eq");
        assert!(hist.same_as(&histc), "bad same as");

        hist.undo();
        hist.update(Move::new(-1, -1, (0, 0)));
        assert_ne!(hist, histc, "bad eq");
        assert!(!hist.same_as(&histc), "bad same as");
    }

    fn new_history<I: IntoIterator<Item = IMove>>(iter: I) -> IHistory {
        let mut hist = IHistory::new();
        iter.into_iter().for_each(|i| hist.update(i));
        hist
    }
}

#[cfg(test)]
mod multi_test {
    use super::*;

    type IMoveVec = Vec<Move<i32>>;
    type IMultiHistory = MultiHistory<i32>;

    #[test]
    fn update_undo_redo() {
        let moves = (0..10)
            .map(|i| vec![Move::new(i, i, (i as _, i as _)); 2])
            .collect::<Vec<_>>();
        let mut hist = new_history(moves.clone());
        assert_eq!(hist.loc, Where::Back, "bad loc");

        for mvs in moves.iter().cloned().rev() {
            assert_eq!(hist.undo().cloned(), Some(mvs), "bad undo");
        }
        assert_eq!(hist.loc, Where::Front, "bad loc");
        assert_eq!(hist.undo(), None, "bad undo");
        for mvs in moves.iter().cloned() {
            assert_eq!(hist.redo().cloned(), Some(mvs), "bad redo");
        }
        assert_eq!(hist.loc, Where::Back, "bad loc");
        assert_eq!(hist.redo(), None, "bad redo");
        assert_eq!(hist, new_history(moves.clone()));

        let mvs = vec![Move::new(0, 0, (0, 0)); 2];
        hist.undo();
        assert_eq!(hist.loc, Where::Middle, "bad loc");
        hist.update(mvs.clone());
        assert_eq!(hist.redo(), None, "bad redo");

        assert_eq!(hist.undo(), Some(&mvs), "bad undo");
        assert_eq!(hist.redo(), Some(&mvs), "bad redo");

        for _ in moves.iter() {
            hist.undo();
        }
        let mvs = vec![Move::new(0, 0, (0, 0)); 2];
        assert_eq!(hist.loc, Where::Front);
        hist.update(mvs.clone());
        assert_eq!(hist.redo(), None, "bad redo");

        assert_eq!(hist.undo(), Some(&mvs), "bad undo");
        assert_eq!(hist.undo(), None, "bad undo");

        assert_eq!(hist.redo(), Some(&mvs), "bad redo");
        assert_eq!(hist.redo(), None, "bad redo");
    }

    #[test]
    fn clone_same_eq() {
        let mut hist = new_history((0..10).map(|i| vec![Move::new(i, i, (i as _, i as _)); 2]));
        let mut histc = hist.clone();
        assert_eq!(hist, histc, "bad clone or eq");
        assert!(hist.same_as(&histc), "bad same as");

        histc.undo();
        assert_ne!(hist, histc, "bad eq");
        assert!(hist.same_as(&histc), "bad same as");

        histc.redo();
        assert_eq!(hist, histc, "bad eq");
        assert!(hist.same_as(&histc), "bad same as");

        histc.undo();
        histc.update(vec![Move::new(-1, -1, (0, 0)); 2]);
        assert_ne!(hist, histc, "bad eq");
        assert!(!hist.same_as(&histc), "bad same as");

        histc.undo();
        histc.update(vec![Move::new(9, 9, (9, 9)); 2]);
        hist.undo();
        assert_ne!(hist, histc, "bad eq");
        assert!(hist.same_as(&histc), "bad same as");

        hist.redo();
        assert_eq!(hist, histc, "bad eq");
        assert!(hist.same_as(&histc), "bad same as");

        hist.undo();
        hist.update(vec![Move::new(-1, -1, (0, 0)); 2]);
        assert_ne!(hist, histc, "bad eq");
        assert!(!hist.same_as(&histc), "bad same as");
    }

    fn new_history<I: IntoIterator<Item = IMoveVec>>(iter: I) -> IMultiHistory {
        let mut hist = IMultiHistory::new();
        iter.into_iter().for_each(|i| hist.update(i));
        hist
    }
}
