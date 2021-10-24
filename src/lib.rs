
use std::cmp::PartialOrd;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

type NodeRef<Key, Payload> = Rc<RefCell<Node<Key, Payload>>>;
type NodeWeak<Key, Payload> = Weak<RefCell<Node<Key, Payload>>>;

pub struct Content<Key, Payload> {
    key: Key,
    payload: Payload
}

struct Node<Key: PartialOrd, Payload> {
    content: Rc<Content<Key, Payload>>,
    parent: Option<NodeWeak<Key, Payload>>,
    left_child: Option<NodeRef<Key, Payload>>,
    right_child : Option<NodeRef<Key, Payload>>
}

enum FindOrInsert<Key: PartialOrd, Payload> {
    Next(NodeRef<Key, Payload>),
    InsertLeft,
    InsertRight,
    Current
}

impl<Key: PartialOrd, Payload> Node<Key, Payload> {

    fn find_or_insert_step(&self, search: &Key) -> FindOrInsert<Key, Payload> {
        if self.content.key < *search {
            if let Some(right) = &self.right_child {
                return FindOrInsert::Next(right.clone());
            }

            return FindOrInsert::InsertRight;
        }
        else if *search < self.content.key {
            if let Some(left) = &self.left_child {
                return FindOrInsert::Next(left.clone());
            }

            return FindOrInsert::InsertLeft;
        }

        // The current node is the intended one, so we will tell them to
        // insert the payload here if desired
        return FindOrInsert::Current;
    }

    fn fall(mut current_node: NodeRef<Key, Payload>) -> NodeRef<Key, Payload> {
        loop {
            let check_left = current_node.borrow().left_child.clone();
            if let Some(left) = check_left {
                current_node = left;
            } else {
                return current_node;
            }
        }
    }

    fn climb(mut from_node: NodeRef<Key, Payload>) -> Option<NodeRef<Key, Payload>> {
        loop {
            let check_to_node = from_node.borrow().parent.clone();
            if let Some(opt_to_node) = check_to_node {
                if let Some(to_node) = opt_to_node.upgrade() {
                    let right_child_opt = to_node.borrow().right_child.clone();
                    if let Some(right_child) = right_child_opt {
                        if Rc::ptr_eq(&right_child, &from_node) {
                            from_node = to_node;
                            continue;
                        }
                    }

                    return Some(to_node);
                }
            }

            return None;
        }
    }

    fn new(key: Key, payload: Payload, parent: Option<NodeWeak<Key, Payload>>)
    -> NodeRef<Key, Payload> {
        return Rc::new(
            RefCell::new(
                Node{
                    content: Rc::new(Content{key: key, payload: payload}),
                    parent: parent,
                    left_child: None,
                    right_child: None
                }
            )
        )
    }
}

pub struct BinarySearchTree<Key: PartialOrd, Payload> {
    root: Option<NodeRef<Key, Payload>>
}

impl<Key: PartialOrd + std::fmt::Display, Payload: std::fmt::Display> BinarySearchTree<Key, Payload> {

    pub fn new() -> BinarySearchTree<Key, Payload> {
        return BinarySearchTree{
            root: None
        };
    }

    pub fn insert(&mut self, key: Key, payload: Payload) -> InsertionResult<Key, Payload> {

        if let Some(mut node) = self.root.clone() {
            loop {
                let next = (*node).borrow().find_or_insert_step(&key);
                match next {
                    FindOrInsert::Next(n) => {
                        node = n;
                    },
                    FindOrInsert::InsertLeft => {
                        let new_node = Node::new(key, payload, Some(Rc::downgrade(&node)));
                        node.borrow_mut().left_child = Some(new_node.clone());
                        return InsertionResult{
                            inserted: true,
                            iterator: BSTIterator{ node: Some(new_node) }
                        };
                    },
                    FindOrInsert::InsertRight => {
                        let new_node = Node::new(key, payload, Some(Rc::downgrade(&node)));
                        node.borrow_mut().right_child = Some(new_node.clone());
                        return InsertionResult{
                            inserted: true,
                            iterator: BSTIterator{ node: Some(new_node) }
                        };
                    },
                    FindOrInsert::Current => {
                        return InsertionResult{
                            inserted: false,
                            iterator: BSTIterator{ node: Some(node) }
                        };
                    }
                }
            }
        } else {
            self.root = Some(Node::new(key, payload, None));
            return InsertionResult{
                inserted: true,
                iterator: BSTIterator{ node: self.root.clone() }
            };
        }
    }

    pub fn iter(&self) -> BSTIterator<Key, Payload> {
        if let Some(root) = &self.root {
            return BSTIterator{ node: Some(Node::fall(root.clone())) };
        } else {
            return BSTIterator{ node: None };
        }
    }

    pub fn print_root(&self) {
        if let Some(root) = &self.root {
            println!(
                "root: key: {} | value: {}",
                &root.borrow().content.key,
                &root.borrow().content.payload
            );
        } else {
            println!("There is no root!");
        }
    }
}

pub struct BSTIterator<Key: PartialOrd + std::fmt::Display, Payload> {
    node: Option<NodeRef<Key, Payload>>
}

impl<Key: PartialOrd + std::fmt::Display, Payload> Iterator for BSTIterator<Key, Payload> {
    type Item = Rc<Content<Key, Payload>>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_node) = self.node.clone() {
            let result = current_node.borrow().content.clone();
            if let Some(right) = current_node.borrow().right_child.clone() {
                self.node = Some(Node::fall(right));
            } else {
                self.node = Node::climb(current_node.clone());
            }

            return Some(result);
        }

        return None;
    }
}

pub struct InsertionResult<Key: PartialOrd + std::fmt::Display, Payload> {
    inserted: bool,
    iterator: BSTIterator<Key, Payload>
}

#[cfg(test)]
mod tests {
    use crate::BinarySearchTree;

    #[test]
    fn it_works() {
        let mut tree = BinarySearchTree::<String, String>::new();
        assert!(tree.insert(String::from("hello"), String::from("there")).inserted);

        let general_kenobi_insert = tree.insert(String::from("General"), String::from("Kenobi"));
        assert!(general_kenobi_insert.inserted);
        if let Some(node) = &general_kenobi_insert.iterator.node {
            assert!(node.borrow().content.key == "General");
        }

        tree.insert(String::from("Anakin"), String::from("Skywalker"));
        tree.insert(String::from("Rey"), String::from("Tatooine"));
        tree.insert(String::from("Manda"), String::from("lorian"));

        tree.print_root();

        println!("Print in order...");
        for n in tree.iter() {
            println!("key: {} | value: {}", n.key, n.payload);
        }
    }
}
