use std::cell::RefCell;
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

impl Item {
  pub fn FullTag(&self, stopper: Rc<RefCell<Item>>) -> String {
    if self.parent.upgrade().is_none()
      || self
        .parent
        .upgrade()
        .unwrap()
        .borrow()
        .parent
        .upgrade()
        .is_none()
      || Rc::ptr_eq(&self.parent.upgrade().clone().unwrap(), &stopper)
    {
      String::from(&self.tag)
    } else {
      format!(
        "{}::{}",
        self.parent.upgrade().unwrap().borrow().FullTag(stopper),
        self.tag
      )
    }
  }
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

  // set a @value of Item with name @name.
  // newly create the item if it doesn't exist.
  // panic if lookup fails.
  pub fn Set(&self, name: &str, value: &str) {
    if let Some(item) = self.lookup(name, true) {
      item.borrow_mut().value = String::from(value);
    } else {
      panic!()
    }
  }

  // set a @value of Item with name @name.
  // if the value already exist, it does nothing.
  pub fn CndSet(&self, name: &str, value: &str) {
    if let Some(item) = self.lookup(name, true) {
      if item.borrow().value.len() == 0 {
        item.borrow_mut().value = String::from(value);
      }
    } else {
      panic!()
    }
  }

  // delete all direct children of @name, having @value.
  pub fn Clear(&self, name: &str, value: &str) {
    if let Some(top) = self.lookup(name, false) {
      let mut prev = top.borrow().child.clone();
      let mut item = top.borrow().child.clone();
      let mut tmp;
      while !item.is_none() {
        if item.clone().unwrap().borrow().value == value {
          if top.borrow().child.clone().unwrap().borrow().tag == item.clone().unwrap().borrow().tag
          {
            // first child, so change head
            top.borrow_mut().child = item.clone().unwrap().borrow().next.clone();
          }
          tmp = item;
          item = tmp.clone().unwrap().borrow().next.clone();
          prev.clone().unwrap().borrow_mut().next = item.clone();
          tmp.clone().unwrap().borrow_mut().next = None;
          tmp.clone().unwrap().borrow_mut().parent = Weak::new();
        } else {
          prev = item.clone();
          item = item.clone().unwrap().borrow().next.clone();
        }
      }
    }
  }

  // delete all direct children of @name
  pub fn ClearForce(&self, name: &str) {
    if let Some(top) = self.lookup(name, false) {
      let mut prev = top.borrow().child.clone();
      let mut item = top.borrow().child.clone();
      let mut tmp;
      while !item.is_none() {
        if top.borrow().child.clone().unwrap().borrow().tag == item.clone().unwrap().borrow().tag {
          // first child, so change head
          top.borrow_mut().child = item.clone().unwrap().borrow().next.clone();
        }
        tmp = item;
        item = tmp.clone().unwrap().borrow().next.clone();
        prev.clone().unwrap().borrow_mut().next = item.clone();
        tmp.clone().unwrap().borrow_mut().next = None;
        tmp.clone().unwrap().borrow_mut().parent = Weak::new();
      }
    }
  }

  // make tree of @oldname head child of @_oldname
  // the very item with @oldname is deleted
  // @oldname, @newname: full path
  pub fn MoveSubTree(&self, oldname: &str, _newname: &str) {
    if String::from(oldname).len() == 0 {
      return;
    }
    if oldname == _newname || format!("{}::", oldname) == _newname {
      return;
    }
    let mut top = self.lookup(oldname, false);
    if top.is_none() {
      return;
    }
    let oldroot = top.clone();
    let stopper = top.clone();
    let mut newname = String::from(_newname);
    if newname.len() != 0 {
      newname.push_str(_newname);
    }

    top.clone().unwrap().borrow_mut().value.clear();
    top = top.clone().unwrap().borrow().child.clone();
    // XXX this line removes reference to its head child,
    // which make refcnt of the child 0, so things would go bad ...?
    //stopper.clone().unwrap().borrow_mut().child = None;

    while !top.is_none() {
      if !top.clone().unwrap().borrow().child.is_none() {
        top = top.clone().unwrap().borrow().child.clone();
        continue;
      }
      while !top.is_none() && top.clone().unwrap().borrow().next.is_none() {
        // finished deleting top's other brothers
        self.Set(
          &format!(
            "{}{}",
            newname,
            top
              .clone()
              .unwrap()
              .borrow()
              .FullTag(oldroot.clone().unwrap())
          ),
          &top.clone().unwrap().borrow().value,
        );
        let tmp = top.clone();
        top = top.unwrap().borrow().parent.upgrade();
        tmp.clone().unwrap().borrow_mut().next = None;
        tmp.clone().unwrap().borrow_mut().parent = Weak::new();
        if !top.is_none() && Rc::ptr_eq(&top.clone().unwrap(), &stopper.clone().unwrap()) {
          return;
        }
      }

      self.Set(
        &format!(
          "{}{}",
          newname,
          top
            .clone()
            .unwrap()
            .borrow()
            .FullTag(oldroot.clone().unwrap())
        ),
        &top.clone().unwrap().borrow().value,
      );
      let tmp = top.clone();
      if !top.is_none() {
        top = top.unwrap().borrow().next.clone();
      }
      tmp.clone().unwrap().borrow_mut().next = None;
      tmp.clone().unwrap().borrow_mut().parent = Weak::new();
    }
  }

  pub fn Find(&self, name: &str, default: &str) -> String {
    if let Some(item) = self.lookup(name, false) {
      if item.borrow().value.len() != 0 {
        return String::from(&item.borrow().value);
      }
    }
    String::from(default)
  }

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
    config.lookup("A", true).unwrap();
    config.lookup("B", true).unwrap();
    config.lookup("A::AA", true).unwrap();
    config.lookup("B::BB", true).unwrap();
    config.lookup("B::BB::BBB1", true).unwrap();
    let bbb2 = config.lookup("B::BB::BBB2", true).unwrap();
    assert_eq!(bbb2.borrow().parent.upgrade().unwrap().borrow().tag, "BB");
    assert_eq!(bbb2.borrow().next.clone().unwrap().borrow().tag, "BBB1");
  }

  #[test]
  pub fn test_set() {
    let config = super::Configuration::new();
    config.lookup("A", true).unwrap();
    config.lookup("A::AA", true).unwrap();
    config.Set("A::AA", "30");
    assert_eq!(config.Find("A::AA", ""), String::from("30"));

    // set value of not existing item.
    config.Set("B::BB", "40");
    assert_eq!(config.Find("B::BB", ""), String::from("40"));

    // conditional set value of already set item.
    config.CndSet("A::AA", "50");
    assert_eq!(config.Find("A::AA", ""), String::from("30"));

    // find not existing item's value
    assert_eq!(config.Find("Not::Exist", ""), String::new());
  }

  #[test]
  pub fn test_clear() {
    let config = super::Configuration::new();
    config.Set("A::AA", "10");
    config.Set("A::AB", "xx");
    config.Set("A::AC", "30");
    config.Set("A::AD", "xx");
    config.Set("B", "0");

    // delete A's direct children having value "xx"
    config.Clear("A", "xx");
    assert_eq!(config.Find("A::AA", ""), "10");
    assert_eq!(config.Find("A::AB", ""), "");
    assert_eq!(config.Find("A::AC", ""), "30");
    assert_eq!(config.Find("A::AD", ""), "");

    // delete A's direct children
    config.Set("A::AA", "10");
    config.Set("A::AB", "xx");
    config.Set("A::AC", "30");
    config.Set("A::AD", "xx");
    config.ClearForce("A");
    assert_eq!(config.Find("A::AA", ""), "");
    assert_eq!(config.Find("A::AB", ""), "");
    assert_eq!(config.Find("A::AC", ""), "");
    assert_eq!(config.Find("A::AD", ""), "");

    // do nothing when the item doesn't exist
    config.Clear("X", "");
    config.ClearForce("X");
    assert_eq!(config.Find("X", ""), "");
  }

  #[test]
  pub fn test_fulltag() {
    let config = super::Configuration::new();
    let node = config.lookup("A::AB::AAA::AAAB::AAAAA", true).unwrap();
    assert_eq!(node.clone().borrow().tag, "AAAAA");
    assert_eq!(
      node.borrow().FullTag(config.root.clone().unwrap()),
      "A::AB::AAA::AAAB::AAAAA"
    );
    let aaa = config.lookup("A::AB::AAA", false).unwrap();
    assert_eq!(node.borrow().FullTag(aaa), "AAAB::AAAAA");
  }

  #[test]
  pub fn test_MoveSubTree() {
    let config = super::Configuration::new();
    config.Set("A", "A");
    config.Set("A::AA0", "AA0");
    config.Set("A::AA1", "AA1");
    config.Set("A::AA2", "AA2");
    config.Set("A::AA1::AAA0", "AAA0");
    config.Set("A::AA1::AAA1", "AAA1");
    config.Set("A::AA1::AAA1::AAAA0", "AAAA0");
    config.Set("A::AA1::AAA1::AAAA1", "AAAA1");
    config.Set("B", "B");
    config.Set("C", "C");

    config.MoveSubTree("A::AA1", "");

    config.lookup("AAA0", false).unwrap();
    config.lookup("AAA1", false).unwrap();
    config.lookup("AAA1::AAAA0", false).unwrap();
    config.lookup("AAA1::AAAA1", false).unwrap();
  }
}
