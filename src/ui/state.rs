use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::profile::validate::{validate, Issue, ValidProfile};
use crate::profile::Profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Edit,
    Replace,
}

type Listener = Rc<dyn Fn(&Profile, &[Issue], Change)>;

pub struct AppState {
    pub profile: Profile,
    pub issues: Vec<Issue>,
    pub valid: Option<ValidProfile>,
    pub profile_path: Option<PathBuf>,
    listeners: Vec<Listener>,
}

pub type Shared = Rc<RefCell<AppState>>;

pub fn new_shared(profile: Profile) -> Shared {
    let (issues, valid) = validate(&profile);
    Rc::new(RefCell::new(AppState { profile, issues, valid, profile_path: None, listeners: Vec::new() }))
}

pub fn on_change(s: &Shared, f: impl Fn(&Profile, &[Issue], Change) + 'static) {
    s.borrow_mut().listeners.push(Rc::new(f));
}

pub fn update(s: &Shared, f: impl FnOnce(&mut Profile)) {
    let mut next = s.borrow().profile.clone();
    f(&mut next);
    if next == s.borrow().profile {
        return;
    }
    set(s, next, Change::Edit);
}

pub fn replace(s: &Shared, profile: Profile) {
    set(s, profile, Change::Replace);
}

fn set(s: &Shared, profile: Profile, change: Change) {
    let (issues, valid) = validate(&profile);
    let listeners = {
        let mut st = s.borrow_mut();
        st.profile = profile.clone();
        st.issues = issues.clone();
        st.valid = valid;
        st.listeners.clone()
    };
    for l in listeners {
        l(&profile, &issues, change);
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
