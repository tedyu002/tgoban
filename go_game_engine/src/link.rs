use std::sync::Arc;
use std::sync::Weak;

pub struct Node<T> {
    pub data: T,
    parent: Option<Weak<Node<T>>>,
    children: Vec<Arc<Node<T>>>,
}

pub struct Tree<T> {
    root: Arc<Node<T>>,

    /// The node to be grown, same as the git branch HEAD
    pub head: Arc<Node<T>>,
}

impl<T> Tree<T> {
    pub fn new(data: T) -> Tree<T> {
        let node = Arc::new(Node::<T> {
            data,
            parent: None,
            children: Vec::new(),
        });

        Tree::<T> {
            root: node.clone(),
            head: node,
        }
    }

    pub fn grow(&mut self, data: T) {
        let node = Arc::new(Node::<T> {
            data,
            parent: Some(Arc::downgrade(&self.head)),
            children: Vec::new(),
        });

        self.head = node;
    }
}
