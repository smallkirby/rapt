use crate::fetcher;
use crate::slist;
use crate::source;

pub fn do_update() {
  log::trace!("do_update()");
  let mut package_items = vec![];

  let sources = match slist::parseSourceFile("sources.list") {
    Ok(_items) => _items,
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  };

  for source in sources {
    let raw_index = match fetcher::fetchIndex(&source) {
      Ok(_raw_index) => _raw_index,
      Err(msg) => {
        println!("{}", msg);
        return;
      }
    };
    match source::SourcePackage::from_row(&raw_index) {
      Ok(mut _items) => {
        log::info!("fetched {} packages.", _items.len());
        package_items.append(&mut _items);
      }
      Err(msg) => {
        println!("{}", msg);
        return;
      }
    }
  }
}
