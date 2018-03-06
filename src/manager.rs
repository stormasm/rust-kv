// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use std::collections::{
    BTreeMap,
};

use std::collections::btree_map::{
    Entry,
};

use std::path::{
    Path,
    PathBuf,
};

use std::fs;

use std::sync::{
    Arc,
    Mutex,
    RwLock,
};

use error::{
    Error,
};

use store::{
    Store
};

use config::{
    Config
};

/// A process is only permitted to have one open handle to each database. This manager
/// exists to enforce that constraint: don't open databases directly.
pub struct Manager {
    stores: Mutex<BTreeMap<PathBuf, Arc<RwLock<Store>>>>,
}

impl Manager {
    /// Create a new store manager
    pub fn new() -> Manager {
        Manager {
            stores: Mutex::new(Default::default()),
        }
    }

    /// Return the open store at `path`, returning `None` if it has not already been opened.
    pub fn get<'p, P>(&self, path: P) -> Result<Option<Arc<RwLock<Store>>>, Error>
    where P: Into<&'p Path>
    {
        let canonical = path.into().canonicalize()?;
        Ok(self.stores.lock().unwrap().get(&canonical).cloned())
    }

    /// Return the open store at cfg.path, or create it using the given config.
    pub fn open(&mut self, cfg: Config) -> Result<Arc<RwLock<Store>>, Error>{
        let _ = fs::create_dir_all(&cfg.path);
        let canonical = cfg.path.as_path().canonicalize()?;
        let mut map = self.stores.lock().unwrap();
        Ok(match map.entry(canonical) {
            Entry::Occupied(e) => e.get().clone(),
            Entry::Vacant(e) => {
                let k = Arc::new(RwLock::new(Store::new(cfg)?));
                e.insert(k).clone()
            }
        })
    }
}

#[cfg(test)]
mod test {
    extern crate tempdir;

    use self::tempdir::TempDir;
    use std::fs;

    use super::*;

    /// Test that the manager will return the same Handle instance each time for each path.
    #[test]
    fn test_same() {
        let root = TempDir::new("test_same").expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let mut manager = Manager::new();

        let p = root.path();
        assert!(manager.get(p).expect("success").is_none());

        let created_arc = manager.open(Config::default(p)).expect("created");
        let fetched_arc = manager.get(p).expect("success").expect("existed");
        assert!(Arc::ptr_eq(&created_arc, &fetched_arc));
    }
}
