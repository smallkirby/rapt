use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

type LinkNode = Option<Rc<RefCell<Item>>>;

#[derive(Debug, Default)]
pub struct Configuration {
  root: LinkNode,
}

// represent list in same hierarchy
pub struct Item {
  parent: Weak<RefCell<Item>>,
  child: LinkNode,
  next: LinkNode,
  value: String,
  tag: String,
}

impl std::fmt::Debug for Item {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let parent_name = if let Some(p) = self.parent.upgrade() {
      String::from(&p.borrow().tag)
    } else {
      String::from("None")
    };
    let child_name = if let Some(p) = &self.child {
      String::from(&p.borrow().tag)
    } else {
      String::from("None")
    };
    let next_name = if let Some(p) = &self.next {
      String::from(&p.borrow().tag)
    } else {
      String::from("None")
    };
    write!(
      f,
      "ITEM {{tag: {:?}, value: {:?}, parent: {:?}, child: {:?}, next: {:?}}}",
      self.tag, self.value, parent_name, child_name, next_name,
    );
    Ok(())
  }
}

impl Configuration {
  pub fn new() -> Configuration {
    Configuration {
      root: Some(Rc::new(RefCell::new(Item {
        parent: Weak::new(),
        child: None,
        next: None,
        value: String::from(""),
        tag: String::from(""),
      }))),
    }
  }

  //pub fn set(&mut self, name: &str, value: &str) {
  //  if let Some(item) = self.lookup(name, true) {
  //    //item.value = value;
  //  }
  //}

  // put child to the head child of @parent
  pub fn push_child(&self, parent: Rc<RefCell<Item>>, val: &str, tag: &str) -> LinkNode {
    let child = Rc::new(RefCell::new(Item {
      parent: Rc::downgrade(&parent),
      child: None,
      next: parent.borrow().child.clone(),
      value: String::from(val),
      tag: String::from(tag),
    }));
    parent.borrow_mut().child = Some(child.clone());
    Some(child)
  }

  // find the direct child with @tag
  pub fn lookup_child(&self, parent: Rc<RefCell<Item>>, tag: &str, create: bool) -> LinkNode {
    let mut cur_item = parent.borrow().child.clone();
    loop {
      match cur_item {
        Some(item) => {
          if item.borrow().tag == tag {
            return Some(item);
          } else {
            cur_item = item.borrow().next.clone();
            continue;
          }
        }
        None => break,
      }
    }

    if !create {
      None
    } else {
      return self.push_child(parent.clone(), "", tag);
    }
  }

  // recursive lookup of Configuration.root
  pub fn lookup(&self, name: &str, create: bool) -> LinkNode {
    if name.len() == 0 {
      // terminator
      return self.root.clone().unwrap().borrow().child.clone();
    };
    let mut item = self.root.clone().unwrap();
    for tag in name.split("::").collect::<Vec<_>>() {
      if let Some(_item) = self.lookup_child(item, tag, create) {
        item = _item;
      } else {
        // failed to find the item, even to create it.
        return None;
      }
    }
    return Some(item);
  }
}

#[cfg(test)]
mod test {
  #[test]
  pub fn configuration_create_one_child() {
    let config = super::Configuration::new();
    let n0 = config
      .lookup_child(config.root.clone().unwrap(), "n0", true)
      .unwrap();
    let n1 = config.lookup_child(n0, "n1", true).unwrap();
    assert_eq!(n1.borrow().tag, "n1");
  }

  #[test]
  pub fn test_lookup_0() {
    let config = super::Configuration::new();
    config.lookup("A", true);
    config.lookup("B", true);
    config.lookup("C", true);
  }

  #[test]
  pub fn test_lookup_1() {
    let config = super::Configuration::new();
    config.lookup("A", true);
    config.lookup("B", true);
    config.lookup("A::AA", true);
    config.lookup("B::BB", true).unwrap();
    config.lookup("B::BB::BBB1", true).unwrap();
    let bbb2 = config.lookup("B::BB::BBB2", true).unwrap();
    assert_eq!(bbb2.borrow().parent.upgrade().unwrap().borrow().tag, "BB");
    assert_eq!(bbb2.borrow().next.clone().unwrap().borrow().tag, "BBB1");
  }
}
