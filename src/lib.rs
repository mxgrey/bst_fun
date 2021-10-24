use std::fmt::Display;
use std::cmp::PartialOrd;

mod arena;
use arena::Arena;

type Storage<Key, Payload> = Arena<Node<Key, Payload>>;

pub struct Content<Key, Payload> {
    key: Key,
    payload: Payload
}

struct Node<Key: PartialOrd, Payload> {
    content: Content<Key, Payload>,
    parent: Option<usize>,
    left_child: Option<usize>,
    right_child : Option<usize>
}

enum TraverseTowards {
    Next(usize),
    InsertLeft,
    InsertRight,
    Current
}

impl<Key: PartialOrd, Payload> Node<Key, Payload> {

    fn traverse_towards<Query>(&self, search: &Query) -> TraverseTowards
    where
        Key: std::borrow::Borrow<Query> + PartialOrd<Query>,
        Query: PartialOrd<Key> + ?Sized
    {
        if self.content.key < *search {
            if let Some(right) = self.right_child {
                return TraverseTowards::Next(right);
            }

            return TraverseTowards::InsertRight;
        }
        else if *search < self.content.key {
            if let Some(left) = self.left_child {
                return TraverseTowards::Next(left);
            }

            return TraverseTowards::InsertLeft;
        }

        // The current node is the intended one, so we will tell them to
        // insert the payload here if desired
        return TraverseTowards::Current;
    }

    fn fall_min(
        storage: &Storage<Key, Payload>,
        mut current_node_index: usize
    ) -> usize {
        loop {
            let current_node = storage.view(current_node_index);
            if let Some(left) = current_node.left_child {
                current_node_index = left;
            } else {
                return current_node_index;
            }
        }
    }

    fn fall_max(
        storage: &Storage<Key, Payload>,
        mut current_node_index: usize
    ) -> usize {
        loop {
            let current_node = storage.view(current_node_index);
            if let Some(right) = current_node.right_child {
                current_node_index = right;
            } else {
                return current_node_index;
            }
        }
    }

    fn climb(
        storage: &Storage<Key, Payload>,
        mut from_node_index: usize
    ) -> Option<usize> {
        loop {
            let from_node = storage.view(from_node_index);
            let check_to_node_index = from_node.parent;
            if let Some(to_node_index) = check_to_node_index {
                let to_node = storage.view(to_node_index);
                if let Some(right_child_index) = to_node.right_child {
                    if right_child_index == from_node_index {
                        from_node_index = to_node_index;
                        continue;
                    }
                }

                return Some(to_node_index);
            }

            return None;
        }
    }

    fn new(
        storage: &mut Storage<Key, Payload>,
        key: Key,
        payload: Payload,
        parent: Option<usize>,
    ) -> usize {
        return storage.alloc(
            Node{
                content: Content{key: key, payload: payload},
                parent: parent,
                left_child: None,
                right_child: None
            }
        );
    }
}

pub struct BinarySearchTree<Key: PartialOrd, Payload> {
    storage: Storage<Key, Payload>,
    root: Option<usize>,
}

impl<'g, Key: PartialOrd + Display, Payload: Display> BinarySearchTree<Key, Payload> {

    pub fn new() -> BinarySearchTree<Key, Payload> {
        return BinarySearchTree{
            storage: Arena::new(),
            root: None,
        };
    }

    pub fn insert(&'g mut self, key: Key, payload: Payload) -> InsertionResult<'g, Key, Payload> {

        if let Some(mut node) = self.root {
            loop {
                let next = self.storage.view(node).traverse_towards(&key);
                match next {
                    TraverseTowards::Next(n) => {
                        node = n;
                    },
                    TraverseTowards::InsertLeft => {
                        let new_left_child =
                            Some(Node::new(&mut self.storage, key, payload, Some(node)));

                        self.storage.modify(node).left_child = new_left_child;

                        return InsertionResult{
                            inserted: true,
                            iterator: BSTIterator{storage: &self.storage, node: new_left_child }
                        };
                    },
                    TraverseTowards::InsertRight => {
                        let new_right_child =
                            Some(Node::new(&mut self.storage, key, payload, Some(node)));

                        self.storage.modify(node).right_child = new_right_child;

                        return InsertionResult{
                            inserted: true,
                            iterator: BSTIterator{ storage: &self.storage, node: new_right_child }
                        };
                    },
                    TraverseTowards::Current => {
                        return InsertionResult{
                            inserted: false,
                            iterator: BSTIterator{ storage: &self.storage, node: Some(node) }
                        };
                    }
                }
            }
        } else {
            self.root = Some(Node::new(&mut self.storage, key, payload, None));
            return InsertionResult{
                inserted: true,
                iterator: BSTIterator{ storage: &self.storage, node: self.root }
            };
        }
    }

    pub fn remove<Query>(&mut self, query: &Query) -> bool
    where
        Key: std::borrow::Borrow<Query> + PartialOrd<Query>,
        Query: PartialOrd<Key> + ?Sized {
        if let Some(mut node) = self.root {
            loop {
                let next = self.storage.view(node).traverse_towards(&query);
                match next {
                    TraverseTowards::Next(n) => {
                        node = n;
                    },
                    TraverseTowards::Current => {
                        self.remove_node(node);
                        return true;
                    },
                    TraverseTowards::InsertLeft => {
                        return false;
                    },
                    TraverseTowards::InsertRight => {
                        return false;
                    }
                }
            }
        }

        return false;
    }

    fn remove_node(&mut self, node_index: usize) {
        let to_remove = self.storage.view(node_index);
        let check_parent = to_remove.parent;
        let check_left = to_remove.left_child;
        let check_right = to_remove.right_child;
        self.storage.remove(node_index);

        if let Some(parent_index) = check_parent {
            let new_child = self.rebuild_tree(check_left, check_right, Some(parent_index));
            let parent_node = self.storage.modify(parent_index);
            if let Some(old_left) = parent_node.left_child {
                if old_left == node_index {
                    parent_node.left_child = new_child;
                    return;
                }
            }

            if let Some(old_right) = parent_node.right_child {
                if old_right == node_index {
                    parent_node.right_child = new_child;
                    return;
                }
            }

            panic!(
                "Broken tree! Could not find child {} in node {}. left:{:?}, right:{:?}",
                node_index,
                parent_index,
                parent_node.left_child,
                parent_node.right_child
            );
        } else {
            self.root = self.rebuild_tree(check_left, check_right, None);
        }
    }

    fn rebuild_tree(
        &mut self,
        check_left: Option<usize>,
        check_right: Option<usize>,
        new_parent: Option<usize>
    ) -> Option<usize> {
        if let Some(left) = check_left {
            // We will let the left node take the place of its parent
            self.storage.modify(left).parent = new_parent;
            if let Some(right) = check_right {
                // If there was a right node, then we will move it to be a child
                // of the max node in the left subtree.
                let left_max_index = Node::fall_max(&self.storage, left);
                self.storage.modify(left_max_index).right_child = Some(right);
                self.storage.modify(right).parent = Some(left_max_index);
            }

            return Some(left);
        } else if let Some(right) = check_right {
            // We will let the right node take the place of the parent
            self.storage.modify(right).parent = new_parent;
            return Some(right);
        } else {
            // If the removed node has no left or right child, then simply
            // clear its position.
            return None;
        }
    }

    pub fn iter(&'g self) -> BSTIterator<'g, Key, Payload> {
        if let Some(root) = self.root {
            return BSTIterator{ storage: &self.storage, node: Some(Node::fall_min(&self.storage, root)) };
        } else {
            return BSTIterator{ storage: &self.storage, node: None };
        }
    }

    pub fn print_root(&self) {
        if let Some(root) = self.root {
            let root_node = self.storage.view(root);
            println!(
                "root: key: {} | value: {}",
                &root_node.content.key,
                &root_node.content.payload
            );
        } else {
            println!("There is no root!");
        }
    }
}

pub struct BSTIterator<'g, Key: PartialOrd + Display, Payload> {
    storage: &'g Arena<Node<Key, Payload>>,
    node: Option<usize>
}

impl<'g, Key: PartialOrd + Display, Payload> BSTIterator<'g, Key, Payload> {
    pub fn view(&self) -> Option<&'g Content<Key, Payload>> {
        if let Some(node_index) = self.node {
            return Some(&self.storage.view(node_index).content);
        } else {
            return None;
        }
    }
}

impl<'g, Key: PartialOrd + Display, Payload> Iterator for BSTIterator<'g, Key, Payload> {
    type Item = &'g Content<Key, Payload>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_node_index) = self.node {
            let current_node = self.storage.view(current_node_index);
            let result = &current_node.content;
            if let Some(right) = current_node.right_child {
                self.node = Some(Node::fall_min(&self.storage, right));
            } else {
                self.node = Node::climb(&self.storage, current_node_index);
            }

            return Some(result);
        }

        return None;
    }
}

pub struct InsertionResult<'g, Key: PartialOrd + Display, Payload> {
    inserted: bool,
    iterator: BSTIterator<'g, Key, Payload>
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
        if let Some(content) = general_kenobi_insert.iterator.view() {
            assert!(content.key == "General");
            println!("Yes, we have the General!");
        }

        tree.insert(String::from("Anakin"), String::from("Skywalker"));
        tree.insert(String::from("Rey"), String::from("Tatooine"));
        tree.insert(String::from("Manda"), String::from("lorian"));

        tree.print_root();

        println!("Print in order...");
        for n in tree.iter() {
            println!("key: {} | value: {}", n.key, n.payload);
        }

        tree.remove(&String::from("General"));

        println!("After removing General...");

        for n in tree.iter() {
            println!("key: {} | value: {}", n.key, n.payload);
        }
    }
}
