//! Small tracked parts of the application state. Includes **window**, **keys**, **mouse**, and
//! **time** - each of which are stored in the **App**.

pub use self::keys::Keys;
pub use self::mouse::Mouse;
pub use self::time::Time;
pub use self::window::Window;

/// Tracked state related to the focused window.
pub mod window {
    use crate::geom;
    use crate::window;

    /// The default scalar value used for window positioning and sizing.
    pub type DefaultScalar = geom::scalar::Default;

    /// State of the window in focus.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Window {
        /// ID of the window currently in focus.
        pub id: Option<window::Id>,
    }

    impl Window {
        /// Initialise the window state.
        pub fn new() -> Self {
            Window { id: None }
        }

        /// Expects that there will be a `window::Id` (the common case) and **panic!**s otherwise.
        pub fn id(&self) -> window::Id {
            self.id.unwrap()
        }
    }
}

/// Tracked state related to the keyboard.
pub mod keys {
    use crate::event::{Key, ModifiersState};
    use std::collections::HashSet;
    use std::ops::Deref;

    /// The state of the keyboard.
    #[derive(Clone, Debug, Default)]
    pub struct Keys {
        /// The state of the modifier keys as last indicated by winit.
        pub mods: ModifiersState,
        /// The state of all keys as tracked via the nannou App event handling.
        pub down: Down,
    }

    /// The set of keys that are currently pressed.
    #[derive(Clone, Debug, Default)]
    pub struct Down {
        pub(crate) keys: HashSet<Key>,
    }

    impl Deref for Down {
        type Target = HashSet<Key>;
        fn deref(&self) -> &Self::Target {
            &self.keys
        }
    }
}

/// Tracked state related to the mouse.
pub mod mouse {
    use crate::geom::Point2;
    use crate::window;
    use std::collections::HashMap;

    #[doc(inline)]
    pub use crate::event::MouseButton as Button;

    /// The state of the `Mouse` at a single moment in time.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Mouse {
        /// The ID of the last window currently in focus.
        pub window: Option<window::Id>,
        /// *x* position relative to the middle of `window`.
        pub x: f32,
        /// *y* position relative to the middle of `window`.
        pub y: f32,
        /// A map describing the state of each mouse button.
        pub buttons: ButtonMap,
    }

    /// Whether the button is up or down.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum ButtonPosition {
        /// The button is up (i.e. unpressed).
        Up,
        /// The button is down and was originally pressed down at the given `Point2`.
        Down(Point2),
    }

    /// Stores the state of all mouse buttons.
    ///
    /// If the mouse button is down, it stores the position of the mouse when the button was pressed
    #[derive(Clone, Debug, PartialEq)]
    pub struct ButtonMap {
        buttons: HashMap<Button, ButtonPosition>,
    }

    impl Mouse {
        /// Construct a new default `Mouse`.
        pub fn new() -> Self {
            Mouse {
                window: None,
                buttons: ButtonMap::new(),
                x: 0.0,
                y: 0.0,
            }
        }

        /// The position of the mouse relative to the middle of the window in focus..
        pub fn position(&self) -> Point2 {
            [self.x, self.y].into()
        }
    }

    impl ButtonPosition {
        /// If the mouse button is down, return a new one with position relative to the given `xy`.
        pub fn relative_to(self, xy: Point2) -> Self {
            match self {
                ButtonPosition::Down(pos) => {
                    let rel_p = pos - xy;
                    ButtonPosition::Down([rel_p.x, rel_p.y].into())
                }
                button_pos => button_pos,
            }
        }

        /// Is the `ButtonPosition` down.
        pub fn is_down(&self) -> bool {
            match *self {
                ButtonPosition::Down(_) => true,
                _ => false,
            }
        }

        /// Is the `ButtonPosition` up.
        pub fn is_up(&self) -> bool {
            match *self {
                ButtonPosition::Up => true,
                _ => false,
            }
        }

        /// Returns the position at which the button was pressed.
        pub fn if_down(&self) -> Option<Point2> {
            match *self {
                ButtonPosition::Down(xy) => Some(xy),
                _ => None,
            }
        }
    }

    impl ButtonMap {
        /// Returns a new button map with all buttons unpressed
        pub fn new() -> Self {
            ButtonMap {
                buttons: HashMap::with_capacity(5),
            }
        }

        /// Returns a copy of the ButtonMap relative to the given `Point2`
        pub fn relative_to(self, xy: Point2) -> Self {
            Self {
                buttons: self
                    .buttons
                    .iter()
                    .map(|(&button, &point)| (button, point.relative_to(xy)))
                    .collect(),
            }
        }

        /// The state of the left mouse button.
        pub fn left(&self) -> &ButtonPosition {
            &self[Button::Left]
        }

        /// The state of the middle mouse button.
        pub fn middle(&self) -> &ButtonPosition {
            &self[Button::Middle]
        }

        /// The state of the right mouse button.
        pub fn right(&self) -> &ButtonPosition {
            &self[Button::Right]
        }

        /// Sets the `Button` in the `Down` position.
        pub fn press(&mut self, button: Button, xy: Point2) {
            self[button] = ButtonPosition::Down(xy);
        }

        /// Set's the `Button` in the `Up` position.
        pub fn release(&mut self, button: Button) {
            self[button] = ButtonPosition::Up;
        }

        /// An iterator yielding all pressed mouse buttons along with the location at which they
        /// were originally pressed.
        pub fn pressed<'a>(&'a self) -> impl Iterator<Item = (Button, Point2)> + 'a {
            self.buttons
                .iter()
                .filter_map(|(&button, &point)| point.if_down().map(|point| (button, point)))
        }
    }

    impl std::ops::Index<Button> for ButtonMap {
        type Output = ButtonPosition;
        fn index(&self, button: Button) -> &Self::Output {
            self.buttons.get(&button).unwrap_or(&ButtonPosition::Up)
        }
    }

    impl std::ops::IndexMut<Button> for ButtonMap {
        fn index_mut(&mut self, button: Button) -> &mut Self::Output {
            self.buttons.entry(button).or_insert(ButtonPosition::Up)
        }
    }
}

/// Tracked durations related to the App.
pub mod time {
    /// The state of time tracked by the App.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct Time {
        /// The duration since the app started running.
        pub since_start: std::time::Duration,
        /// The duration since the previous update.
        pub since_prev_update: std::time::Duration,
    }

    impl Time {
        /// The number of updates per second if `since_prev_update` were to remain constant
        pub fn updates_per_second(&self) -> f32 {
            if self.since_prev_update.as_secs() > 0 {
                return 0.0;
            }

            let millis = self.since_prev_update.subsec_millis() as f32;

            if millis == 0.0 {
                return std::f32::MAX;
            }

            1000.0 / millis
        }
    }
}
