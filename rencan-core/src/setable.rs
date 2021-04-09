pub struct Mutable<State, Depends> {
    state: State,
    depends: Option<Depends>,
}

impl<State, Depends> Mutable<State, Depends> {
    pub fn new(state: State) -> Self {
        Self { state, depends: None }
    }
    pub fn change(self, f: impl FnOnce(State) -> State) -> Self {
        let state = f(self.state);
        Self::new(state)
    }
    pub fn change_with_check(self, new_state: State) -> Self
    where
        State: PartialEq<State>,
    {
        if self.state == new_state {
            self
        } else {
            Self::new(new_state)
        }
    }
    pub fn change_with_check_in_place(&mut self, new_state: State)
    where
        State: PartialEq<State>,
    {
        if self.state != new_state {
            let new_val = Self::new(new_state);
            *self = new_val;
        }
    }
    pub fn get_depends_or_init(&mut self, f: impl FnOnce(&State) -> Depends) -> &Depends {
        match &self.depends {
            Some(_) => {}
            None => {
                self.depends = Some(f(&self.state));
            }
        }
        self.depends.as_ref().unwrap()
    }
    pub fn is_changed(&self) -> bool {
        self.depends.is_none()
    }
    pub fn state(&self) -> &State {
        &self.state
    }
}
