use std::sync::{Arc, Weak};
use std::cell::RefCell;

pub struct Node<T> {
    pub data: T,
    pub parent: Option<Weak<RefCell<Node<T>>>>,
    pub children: Vec<Arc<RefCell<Node<T>>>>,
}

pub struct Tree<T> {
    root: Arc<RefCell<Node<T>>>,

    /// The node to be grown, same as the git branch HEAD
    pub head: Arc<RefCell<Node<T>>>,
}

impl<T> Tree<T> {
    pub fn new(data: T) -> Tree<T> {
        let node = Arc::new(RefCell::new(Node::<T> {
            data,
            parent: None,
            children: Vec::new(),
        }));

        Tree::<T> {
            root: node.clone(),
            head: node,
        }
    }

    pub fn grow(&mut self, data: T) {
        let node = Arc::new(RefCell::new(Node::<T> {
            data,
            parent: Some(Arc::downgrade(&self.head)),
            children: Vec::new(),
        }));

        self.head.borrow_mut().children.push(node.clone());

        self.head = node;
    }
}
