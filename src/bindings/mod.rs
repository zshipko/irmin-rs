use crate::{Key, Type};
use ocaml_interop::*;

ocaml! {
    fn config(root: String) -> Config;
    fn repo(config: Config) -> Repo;
    fn store_master(repo: Repo) -> Store;
    fn store_mem(store: Store, key: String) -> bool;
    fn store_remove(store: Store, key: String, message: String);
    fn store_find(store: Store, key: String) -> Option<OCamlBytes>;
    fn store_set(store: Store, key: String, value: String, message: String);
}

pub struct Context {
    rt: std::cell::RefCell<OCamlRuntime>,
}

impl Context {
    pub fn new() -> Context {
        let rt = std::cell::RefCell::new(OCamlRuntime::init());
        Context { rt }
    }
}

macro_rules! wrapper {
    ($x: ident) => {
        pub struct $x(BoxRoot<$x>);

        unsafe impl FromOCaml<$x> for $x {
            fn from_ocaml(v: OCaml<'_, $x>) -> Self {
                $x(v.root())
            }
        }

        unsafe impl ToOCaml<$x> for $x {
            fn to_ocaml<'a>(&self, rt: &'a mut OCamlRuntime) -> OCaml<'a, $x> {
                self.0.get(rt)
            }
        }
    };
}

unsafe impl FromOCaml<String> for Key {
    fn from_ocaml(v: OCaml<'_, String>) -> Self {
        let mut bytes = v.as_bytes();
        Key::decode_bin(&mut bytes).expect("Invalid key argument passed to Rust")
    }
}

unsafe impl ToOCaml<String> for Key {
    fn to_ocaml<'a>(&self, rt: &'a mut OCamlRuntime) -> OCaml<'a, String> {
        let mut data = Vec::new();
        self.encode_bin(&mut data)
            .expect("Invalid key argument passed to OCaml");
        data.to_ocaml(rt)
    }
}

wrapper!(Config);

impl Config {
    pub fn new(ctx: &Context, root: impl AsRef<str>) -> Config {
        let cr = &mut ctx.rt.borrow_mut();
        let root = root.as_ref().to_ocaml(cr).root();
        let x: BoxRoot<Config> = config(cr, &root);
        x.to_rust(cr)
    }
}

wrapper!(Repo);

impl Repo {
    pub fn new(ctx: &Context, cfg: &Config) -> Repo {
        let cr = &mut ctx.rt.borrow_mut();
        let cfg = cfg.to_ocaml(cr).root();
        let x: BoxRoot<Repo> = repo(cr, &cfg);
        x.to_rust(cr)
    }
}

wrapper!(Store);

impl Store {
    pub fn master(ctx: &Context, repo: &Repo) -> Store {
        let cr = &mut ctx.rt.borrow_mut();
        let repo = repo.to_ocaml(cr).root();
        let x: BoxRoot<Store> = store_master(cr, &repo);
        x.to_rust(cr)
    }

    pub fn mem(&self, ctx: &Context, key: &Key) -> bool {
        let cr = &mut ctx.rt.borrow_mut();
        let store = self.to_ocaml(cr).root();
        let key = key.to_ocaml(cr).root();
        let x: BoxRoot<bool> = store_mem(cr, &store, &key);
        x.to_rust(cr)
    }

    pub fn find(&self, ctx: &Context, key: &Key) -> Option<String> {
        let cr = &mut ctx.rt.borrow_mut();
        let store = self.to_ocaml(cr).root();
        let key = key.to_ocaml(cr).root();
        let x = store_find(cr, &store, &key);
        x.to_rust(cr)
    }

    pub fn remove(&self, ctx: &Context, key: &Key, msg: impl AsRef<str>) {
        let cr = &mut ctx.rt.borrow_mut();
        let store = self.to_ocaml(cr).root();
        let key = key.to_ocaml(cr).root();
        let info = msg.as_ref().to_ocaml(cr).root();
        let _: BoxRoot<()> = store_remove(cr, &store, &key, &info);
    }

    pub fn set(&self, ctx: &Context, key: &Key, value: impl AsRef<[u8]>, msg: impl AsRef<str>) {
        let cr = &mut ctx.rt.borrow_mut();
        let store = self.to_ocaml(cr).root();
        let key = key.to_ocaml(cr).root();
        let info = msg.as_ref().to_ocaml(cr).root();
        let value = value.as_ref().to_ocaml(cr).root();
        let _: BoxRoot<()> = store_set(cr, &store, &key, &value, &info);
    }
}

#[cfg(test)]
mod tests {
    use crate::bindings::*;

    #[test]
    fn basic() {
        let ctx = Context::new();
        let cfg = Config::new(&ctx, "test123");
        let repo = Repo::new(&ctx, &cfg);
        let master = Store::master(&ctx, &repo);
        let key = Key::new(&["a", "b", "c"]);
        let x = master.find(&ctx, &key);
        assert!(x.is_none());

        master.set(&ctx, &key, "123", "test");

        let x = master.find(&ctx, &key).unwrap();
        assert!(x.as_str() == "123");
    }
}
