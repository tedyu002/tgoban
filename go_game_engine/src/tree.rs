use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub(crate) struct Node<T> {
    pub data: T,
    pub parent: Option<Weak<RefCell<Node<T>>>>,
    pub children: Vec<Rc<RefCell<Node<T>>>>,
}

pub(crate) struct Tree<T> {
    root: Rc<RefCell<Node<T>>>,

    /// The node to be grown, same as the git branch HEAD
    pub head: Rc<RefCell<Node<T>>>,
}

impl<T> Tree<T> {
    pub fn new(data: T) -> Tree<T> {
        let node = Rc::new(RefCell::new(Node::<T> {
            data,
            parent: None,
            children: Vec::new(),
        }));

        Tree::<T> {
            root: node.clone(),
            head: node,
        }
    }

    pub fn access_head<F>(&self, f:F) where
        F: FnOnce(&T) {

        f(&self.head.borrow().data);
    }

    pub fn grow<F>(&mut self, f: F) where
        F: FnOnce(&T) -> T {

        let new_node = Rc::new(RefCell::new(Node::<T> {
            data: f(&(*self.head).borrow().data),
            parent: Some(Rc::downgrade(&self.head)),
            children: Vec::new(),
        }));

        self.head.borrow_mut().children.push(new_node.clone());

        self.head = new_node;
    }

    pub fn remove_head<F>(&mut self, f:F) where
        F: FnOnce(&T) {

        let has_parent = {
            match (*self.head).borrow().parent {
                None => false,
                _ => {
                    true
                },
            }
        };

        if !has_parent {
            return;
        }

        let removed_head = self.head.clone();
        self.head = removed_head.borrow().parent
            .as_ref()
            .unwrap() /* Always success since the parent exist */
            .upgrade()
            .unwrap() /* Always success since the tree holds */;

        let index = {
            self.head.borrow().children.iter().position(|x| {
                Rc::ptr_eq(&x, &removed_head)
            }).unwrap()
        };

        (*self.head).borrow_mut().children.remove(index);

        f(&removed_head.borrow().data);
    }
}
