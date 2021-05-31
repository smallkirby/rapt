use std::rc::Rc;
use std::cell::{Cell, RefCell};

type LinkNode = Option<Rc<RefCell<Node>>>;

#[derive(Debug, Default,)]
pub struct Node {
  next: LinkNode,
  parent: LinkNode,
  child: LinkNode,
  val: String,
}

pub fn push_child(parent: Rc<RefCell<Node>>, val: &str) -> Option<Rc<RefCell<Node>>> {
  let item = Rc::new(RefCell::new(Node {
    parent: Some(parent.clone()),
    child: None,
    next: parent.borrow().child.clone(),
    val: String::from(val),
  }));
  parent.borrow_mut().child = Some(item.clone());
  Some(item)
}

//pub fn lookup_item(parent: Rc<Node>, val: &str) -> Option<Rc<Node>> {
//  let mut item;
//  loop {
//    item = parent.as_ref().
//  }
//}

#[cfg(test)]
mod test {
  use std::rc::Rc;
  use std::cell::{Cell, RefCell};
  #[test]
  pub fn test1() {
    let n0 = Rc::new(RefCell::new(super::Node{
      parent: None,
      child: None,
      next: None,
      val: String::from("n0"),
    }));
    let n1 = super::push_child(n0, "n1").unwrap();
    //println!("{:?}", n1);
  }
}