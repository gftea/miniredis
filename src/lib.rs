pub mod cmd;
pub mod connection;
pub mod database;
pub mod frame;
pub mod  server;


#[inline]
pub fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

// Definition for a binary tree node.
#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }
}
use std::cell::RefCell;
use std::ops::Rem;
use std::rc::Rc;

pub struct Solution;
impl Solution {
    pub fn max_depth(root: Option<Rc<RefCell<TreeNode>>>) -> i32 {
        Self::next_node(root.as_ref(), 0)
    }

    fn next_node(node: Option<&Rc<RefCell<TreeNode>>>, mut num: i32) -> i32 {
        match node {
            None => num,
            Some(rc_node) => {
                num += 1;
                use std::borrow::Borrow; //blanket implementation

                let lmax = Self::next_node((&**rc_node ).borrow().left.as_ref(), num);

                let rmax = Self::next_node(<Rc<RefCell<TreeNode>> as Borrow<RefCell<TreeNode>>>::borrow(rc_node).borrow().right.as_ref(), num);
                lmax.max(rmax)
            }
        }
    }
}

pub fn add_leaf_nodes(node: Option<Rc<RefCell<TreeNode>>>, cnt: &mut i32, max: i32) {
    match node {
        None => (),
        Some(root) => {
            if *cnt >= max {
                return;
            }
            match cnt.rem(99) {
                5 => {
                    root.borrow_mut().left = None;
                    root.borrow_mut().right = Some(Rc::new(RefCell::new(TreeNode::new(*cnt))));
                }
                9 => {
                    root.borrow_mut().left = Some(Rc::new(RefCell::new(TreeNode::new(*cnt))));
                    root.borrow_mut().right = None;
                }
                _ => {
                    root.borrow_mut().left = Some(Rc::new(RefCell::new(TreeNode::new(*cnt))));
                    root.borrow_mut().right = Some(Rc::new(RefCell::new(TreeNode::new(*cnt))));
                }
            }

            *cnt += 2;
            add_leaf_nodes(root.borrow().left.clone(), cnt, max);
            add_leaf_nodes(root.borrow().right.clone(), cnt, max);
        }
    }
}

pub fn generate_binary_tree(max: i32) -> Option<Rc<RefCell<TreeNode>>> {
    let root = Some(Rc::new(RefCell::new(TreeNode::new(0))));
    let mut cnt = 0;
    add_leaf_nodes(root.clone(), &mut cnt, max);

    root
}

fn traverse(root: Option<Rc<RefCell<TreeNode>>>) {
    match root {
        Some(node) => {
            // println!("{node:?}");
            traverse(node.borrow().left.clone());
            traverse(node.borrow().right.clone());
        }
        None => (),
    }
}

#[test]
fn test_tree_generator() {
    let root = generate_binary_tree(100);
    // traverse(root);
    // println!("{root:#?}")
}
