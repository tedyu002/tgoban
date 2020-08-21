use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub(crate) struct Node<T> {
    data: T,
    parent: Option<Weak<RefCell<Node<T>>>>,

    first_child: Option<Rc<RefCell<Node<T>>>>,
    last_child: Option<Weak<RefCell<Node<T>>>>,

    next: Option<Rc<RefCell<Node<T>>>>,
    prev: Option<Weak<RefCell<Node<T>>>>,
}

pub(crate) struct Tree<T> {
    root: Rc<RefCell<Node<T>>>,

    /// The node to be grown, same as the git branch HEAD
    head: Rc<RefCell<Node<T>>>,
}


/// The Rc, RefCell only used internal, no leakage to outer.
unsafe impl<T: Send> Send for Tree<T> {}

impl<T> Tree<T> {
    pub fn new(data: T) -> Tree<T> {
        let node = Rc::new(RefCell::new(Node::<T> {
            data,
            parent: None,
            first_child: None,
            last_child: None,
            prev: None,
            next: None,
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
            first_child: None,
            last_child: None,
            next: None,
            prev: None,
        }));

        {
            let mut head = self.head.borrow_mut();

            if let Some(last_child) = head.last_child.as_ref() {
                new_node.borrow_mut().prev = Some(last_child.clone());

                let last_child = last_child.upgrade().unwrap() /* Can unwrap since the prev holds the next Rc */;
                last_child.borrow_mut().next = Some(new_node.clone());
            }

            head.last_child = Some(Rc::downgrade(&new_node));

            if let None = head.first_child {
                head.first_child = Some(new_node.clone());
            }
        }

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
        let removed_head = removed_head.borrow();

        let parent = removed_head.parent.as_ref().unwrap().upgrade().unwrap();
        let mut parent = parent.borrow_mut();

        { /* Changes the next of prev, if no prev, means the first_child of parent is changed. */
            if let Some(prev) = removed_head.prev.as_ref() {
                match removed_head.next.as_ref() {
                    Some(next) => {
                        prev.upgrade()
                            .unwrap() /* Always success since the root can walk to it */
                            .borrow_mut().next = Some(next.clone());
                    },
                    None => {},
                };
            } else {
                /* Can safe remove the Rc since the Rc is hold by removed_head now */
                parent.first_child = match removed_head.next.as_ref() {
                    Some(next) => {
                        Some(next.clone())
                    },
                    None => {
                        None
                    },
                }
            }
        }

        { /* Changes the prev of next, if no next, means the last_child of parent is changed. */
            if let Some(next) = removed_head.next.as_ref() {
                match removed_head.prev.as_ref() {
                    Some(prev) => {
                        next.borrow_mut().prev = Some(prev.clone());
                    },
                    None => {},
                }
            } else {
                parent.last_child = match removed_head.prev.as_ref() {
                    Some(prev) => {
                        Some(prev.clone())
                    },
                    None => {
                        None
                    },
                }
            }

        }

        self.head = removed_head.parent
            .as_ref()
            .unwrap() /* Always success since the parent exist */
            .upgrade()
            .unwrap() /* Always success since the tree holds */;

        f(&removed_head.data);
    }

    pub fn preorder<F>(&self, mut f: F) where F: FnMut(&T) {
        let iterator: PreIterator<T> = PreIterator::new(&self.root);

        for node in iterator {
            f(&node.borrow().data);
        }
    }
}

struct PreIterator<T> {
    node: Rc<RefCell<Node<T>>>,
    has_next: bool,
}

impl<T> PreIterator<T> {
    fn new(node: &Rc<RefCell<Node<T>>>) -> PreIterator::<T> {
        PreIterator::<T> {
            node: node.clone(),
            has_next: true,
        }
    }
}

impl<T> Iterator for PreIterator<T> {
    type Item = Rc<RefCell<Node<T>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_next {
            return None;
        }

        let current_node = self.node.clone();

        { /* Move to next*/
            let mut current_node = current_node.clone();

            let has_first_child = {
                match current_node.borrow().first_child {
                    None => false,
                    _ => true,
                }
            };

            if has_first_child {
                self.node = current_node.borrow().first_child.as_ref().unwrap().clone();
            } else {
                loop {
                    let find_node = {
                        let current_node_ref = current_node.borrow();

                        if let Some(next) = current_node_ref.next.as_ref() {
                            self.node = next.clone();
                            break;
                        }

                        match current_node_ref.parent.as_ref() {
                            Some(parent) => {
                                parent.upgrade().unwrap()
                            },
                            None => {
                                self.has_next = false;
                                break;
                            }
                        }
                    };

                    current_node = find_node;
                }
            }
        }

        return Some(current_node);
    }
}
