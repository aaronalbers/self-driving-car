use behavior::{Action, Behavior, Priority};
use eeg::{color, Drawable, EEG};
use rlbot;
use std::collections::VecDeque;
use strategy::Context;

/// Run `child` until it returns, then do nothing forever.

#[allow(dead_code)]
pub struct Fuse {
    child: Option<Box<Behavior>>,
}

impl Fuse {
    #[allow(dead_code)]
    pub fn new(child: Box<Behavior>) -> Self {
        Self { child: Some(child) }
    }
}

impl Behavior for Fuse {
    fn name(&self) -> &'static str {
        stringify!(Fuse)
    }

    fn execute(&mut self, _packet: &rlbot::LiveDataPacket, _eeg: &mut EEG) -> Action {
        // `take()` leaves a None behind, so this can only match `Some` once.
        match self.child.take() {
            Some(b) => Action::Call(b),
            None => Action::Yield(Default::default()),
        }
    }
}

/// Do `behavior` forever
#[allow(dead_code)]
pub struct Repeat<B, F>
where
    B: Behavior,
    F: Fn() -> B + Send,
{
    factory: F,
    current: B,
}

impl<B, F> Repeat<B, F>
where
    B: Behavior,
    F: Fn() -> B + Send,
{
    #[allow(dead_code)]
    pub fn new(factory: F) -> Self {
        let current = factory();
        Self { factory, current }
    }
}

impl<B, F> Behavior for Repeat<B, F>
where
    B: Behavior,
    F: Fn() -> B + Send,
{
    fn name(&self) -> &'static str {
        stringify!(Repeat)
    }

    fn execute2(&mut self, ctx: &mut Context) -> Action {
        ctx.eeg
            .draw(Drawable::print(self.current.name(), color::YELLOW));
        match self.current.execute2(ctx) {
            Action::Yield(i) => Action::Yield(i),
            Action::Call(b) => Action::Call(b),
            Action::Return | Action::Abort => {
                ctx.eeg.log("[Repeat] repeating");
                self.current = (self.factory)();
                self.execute2(ctx)
            }
        }
    }
}

/// Run `children` in sequence.
pub struct Chain {
    priority: Priority,
    children: VecDeque<Box<Behavior>>,
}

impl Chain {
    pub fn new(priority: Priority, children: Vec<Box<Behavior>>) -> Self {
        Self {
            priority,
            children: children.into_iter().collect(),
        }
    }
}

impl Behavior for Chain {
    fn name(&self) -> &'static str {
        stringify!(Chain)
    }

    fn priority(&self) -> Priority {
        self.priority
    }

    fn execute2(&mut self, ctx: &mut Context) -> Action {
        ctx.eeg.draw(Drawable::print(
            self.children
                .iter()
                .map(|b| b.name())
                .collect::<Vec<_>>()
                .join(", "),
            color::GREEN,
        ));

        let action = {
            let mut front = match self.children.front_mut() {
                None => return Action::Return,
                Some(b) => b,
            };
            ctx.eeg.draw(Drawable::print(front.name(), color::YELLOW));
            front.execute2(ctx)
        };

        match action {
            Action::Yield(x) => Action::Yield(x),
            Action::Call(b) => {
                ctx.eeg
                    .log(format!("[Chain] replacing head with {}", b.name()));
                self.children[0] = b;
                self.execute2(ctx)
            }
            Action::Return => {
                ctx.eeg.log("[Chain] advancing");
                self.children.pop_front();
                self.execute2(ctx)
            }
            Action::Abort => {
                ctx.eeg.log("[Chain] aborting");
                Action::Abort
            }
        }
    }
}
