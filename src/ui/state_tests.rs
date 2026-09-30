use super::*;
use crate::profile::{Bootloader, Profile};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn update_revalidates_and_notifies_edit() {
    let s = new_shared(Profile::new_default());
    let seen: Rc<RefCell<Vec<(Change, usize)>>> = Rc::default();
    let seen2 = seen.clone();
    on_change(&s, move |_, issues, change| seen2.borrow_mut().push((change, issues.len())));
    update(&s, |p| p.bootloaders.clear());
    assert_eq!(*seen.borrow(), vec![(Change::Edit, 1)]);
    assert!(s.borrow().valid.is_none());
}

#[test]
fn replace_notifies_replace_and_updates_validity() {
    let s = new_shared(Profile::new_default());
    let seen: Rc<RefCell<Vec<Change>>> = Rc::default();
    let seen2 = seen.clone();
    on_change(&s, move |_, _, change| seen2.borrow_mut().push(change));
    let mut p = Profile::new_default();
    p.bootloaders = vec![Bootloader::Limine];
    replace(&s, p.clone());
    assert_eq!(*seen.borrow(), vec![Change::Replace]);
    assert_eq!(s.borrow().profile, p);
    assert!(s.borrow().valid.is_some());
}

#[test]
fn listeners_may_read_and_update_state() {
    let s = new_shared(Profile::new_default());
    let s2 = s.clone();
    on_change(&s, move |_, _, change| {
        let _ = s2.borrow().profile.name.clone();
        if change == Change::Replace {
            update(&s2, |p| p.name = "from-listener".into());
        }
    });
    replace(&s, Profile::new_default());
    assert_eq!(s.borrow().profile.name, "from-listener");
}

#[test]
fn update_that_changes_nothing_does_not_notify() {
    let s = new_shared(Profile::new_default());
    let count = Rc::new(RefCell::new(0));
    let c2 = count.clone();
    on_change(&s, move |_, _, _| *c2.borrow_mut() += 1);
    update(&s, |p| p.name = "void-live".into());
    assert_eq!(*count.borrow(), 0);
}
