use std::vec::Vec;

pub(crate) struct Arena<Node> {
    nodes: Vec<Option<Node>>,
    available: Vec<usize>
}

impl<'a, Node> Arena<Node> {
    pub fn new() -> Arena<Node> {
        return Arena{
            nodes: Vec::new(),
            available: Vec::new()
        };
    }

    pub fn alloc(&mut self, new_node: Node) -> usize {
        if let Some(index) = self.available.pop() {
            self.nodes.insert(index, Some(new_node));
            return index;
        }

        self.nodes.push(Some(new_node));
        return self.nodes.len() - 1;
    }

    pub fn remove(&mut self, index: usize) -> bool {
        let to_erase = &mut self.nodes[index];
        if to_erase.is_some() {
            *to_erase = None;
            self.available.push(index);
            return true;
        }

        return false;
    }

    pub fn view(&'a self, index: usize) -> &'a Node {
        if let Some(node) = &self.nodes[index] {
            return node;
        } else {
            panic!("Requested access to a dead node: {}", index);
        }
    }

    pub fn modify(&'a mut self, index: usize) -> &'a mut Node {
        if let Some(node) = &mut self.nodes[index] {
            return node;
        } else {
            panic!("Requested mutable access to a dead node: {}", index);
        }
    }
}
