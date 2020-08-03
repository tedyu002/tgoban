use std::sync::{Arc, Weak, RwLock};

pub struct Node<T> {
    pub data: T,
    pub parent: Option<Weak<RwLock<Node<T>>>>,
    pub children: Vec<Arc<RwLock<Node<T>>>>,
}

pub struct Tree<T> {
    root: Arc<RwLock<Node<T>>>,

    /// The node to be grown, same as the git branch HEAD
    pub head: Arc<RwLock<Node<T>>>,
}

impl<T> Tree<T> {
    pub fn new(data: T) -> Tree<T> {
        let node = Arc::new(RwLock::new(Node::<T> {
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
        let head_guard = match self.head.read() {
            Ok(guard) => {
                guard
            },
            Err(_) => {
                panic!("Failed to get read lock");
            }
        };

        let head_node = &*head_guard;
        f(&head_node.data);
    }

    pub fn grow<F>(&mut self, f: F) where
        F: FnOnce(&T) -> T {

        let grow_node = {
            let mut head_guard = match self.head.write() {
                Ok(guard) => {
                    guard
                },
                Err(_) => {
                    panic!("Failed to get read lock");
                }
            };

            let head_node = &mut *head_guard;

            let new_node = Arc::new(RwLock::new(Node::<T> {
                data: f(&head_node.data),
                parent: Some(Arc::downgrade(&self.head)),
                children: Vec::new(),
            }));

            head_node.children.push(new_node.clone());

            new_node
        };

        self.head = grow_node;
    }

    pub fn remove_head<F>(&mut self, f:F) where
        F: FnOnce(&T) {
        let parent = {
            let read_guard = match self.head.write() {
                Ok(guard) => {
                    guard
                },
                Err(_) => {
                    panic!("Failed to get write lock");
                }
            };

            let head_node = &*read_guard;

            match &head_node.parent {
                None => {
                    return;
                },
                Some(weak_node) => {
                    weak_node.upgrade().unwrap()
                    /* always successful since the granpa or the root */
                }
            }
        };

        {
            let mut parent_write_guard = match parent.write() {
                Err(_) => {
                    panic!("Failed to get write lock");
                },
                Ok(guard) => {
                    guard
                }
            };

            let parent_node = &mut *parent_write_guard;
            {
                let index = {
                    parent_node.children.iter().position(|x| {
                        Arc::ptr_eq(&x, &self.head)
                    }).unwrap()
                };
                parent_node.children.remove(index);
            }
        }

        let remove_node = self.head.clone();

        {
            let guard = match remove_node.read() {
                Err(_) => {
                    panic!("Failed to get read lock");
                },
                Ok(guard) => {
                    guard
                }
            };

            let remove_node = &*guard;

            f(&remove_node.data);
        }

        self.head = parent;
    }
}
