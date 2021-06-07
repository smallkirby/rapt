use crate::fetcher;
use crate::slist;
use crate::source;

pub fn do_update() {
  log::trace!("do_update()");
  let mut package_items = vec![];

  // read sources.list
  let sources = match slist::parseSourceFile("sources.list") {
    Ok(_items) => _items,
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  };

  // fetch index files and get package items.
  for (ix, source) in sources.iter().enumerate() {
    println!("Get:{} {}", ix, source.info());
    let raw_index = match fetcher::fetchIndex(&source) {
      Ok(_raw_index) => _raw_index,
      Err(msg) => {
        println!("{}", msg);
        return;
      }
    };
    println!("Hit:{} {} [{} B]", ix, source.info(), raw_index.len());
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

  let resolved_items = source::resolve_duplication(&package_items);
}
