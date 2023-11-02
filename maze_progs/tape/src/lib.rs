use std::ops::{Index, IndexMut};

#[derive(Debug, Default, Clone, Copy)]
pub struct Delta<ID, T> {
    pub id: ID,
    pub before: T,
    pub after: T,
    pub burst: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Tape<ID, T> {
    steps: Vec<Delta<ID, T>>,
    i: usize,
}

impl<ID, T> Index<usize> for Tape<ID, T> {
    type Output = Delta<ID, T>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.steps[index]
    }
}

impl<ID, T> IndexMut<usize> for Tape<ID, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.steps[index]
    }
}

impl<ID, T> Tape<ID, T> {
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn end(&mut self) {
        if self.steps.is_empty() {
            panic!("no tape to end because no deltas provided");
        }
        self.i = self.steps.len() - 1;
    }

    pub fn start(&mut self) {
        if self.steps.is_empty() {
            panic!("no tape to start because no deltas provided");
        }
        self.i = 0;
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn cur_step(&self) -> Option<&[Delta<ID, T>]> {
        if self.steps.is_empty() {
            return None;
        }
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    fn peek_next_index(&self) -> usize {
        if self.steps.is_empty() || self.i + self.steps[self.i].burst >= self.steps.len() {
            return self.i;
        }
        self.i + self.steps[self.i].burst
    }

    fn peek_prev_index(&self) -> usize {
        if self.steps.is_empty()
            || self.i == 0
            || self.i.overflowing_sub(self.steps[self.i - 1].burst).1
        {
            return self.i;
        }
        self.i - self.steps[self.i - 1].burst
    }

    pub fn peek_next_delta(&self) -> Option<&[Delta<ID, T>]> {
        if self.i + self.steps[self.i].burst >= self.steps.len() {
            return None;
        }
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn peek_prev_delta(&self) -> Option<&[Delta<ID, T>]> {
        if self.i == 0 || self.i.overflowing_sub(self.steps[self.i - 1].burst).1 {
            return None;
        }
        Some(&self.steps[self.i - self.steps[self.i - 1].burst..self.i])
    }

    pub fn next_delta(&mut self) -> Option<&[Delta<ID, T>]> {
        if self.steps.is_empty() || self.i + self.steps[self.i].burst >= self.steps.len() {
            return None;
        }
        self.i += self.steps[self.i].burst;
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn prev_delta(&mut self) -> Option<&[Delta<ID, T>]> {
        if self.i == 0 || self.i.overflowing_sub(self.steps[self.i - 1].burst).1 {
            return None;
        }
        self.i -= self.steps[self.i - 1].burst;
        Some(&self.steps[self.i..self.i + self.steps[self.i].burst])
    }

    pub fn push_burst(&mut self, steps: &[Delta<ID, T>])
    where
        Delta<ID, T>: Copy,
    {
        if steps.is_empty()
            || steps[0].burst != steps.len()
            || steps[steps.len() - 1].burst != steps.len()
        {
            panic!("ill formed burst input burst");
        }
        for s in steps.iter() {
            self.steps.push(*s);
        }
    }

    pub fn push(&mut self, s: Delta<ID, T>) {
        if s.burst != 1 {
            panic!("single delta has burst length of {}", s.burst);
        }
        self.steps.push(s);
    }

    pub fn at_end(&self) -> bool {
        self.i == self.peek_next_index()
    }

    pub fn at_start(&self) -> bool {
        self.i == self.peek_prev_index()
    }

    pub fn set_prev(&mut self) -> bool {
        let prev = self.i;
        self.i = self.peek_prev_index();
        self.i != prev
    }

    pub fn set_next(&mut self) -> bool {
        let prev = self.i;
        self.i = self.peek_next_index();
        self.i != prev
    }
}
