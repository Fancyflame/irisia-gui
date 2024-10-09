use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::{Rc, Weak},
};

use crate::{
    data_flow::{dirty_flag::DirtyFlag, AsReadWire, ReadWire, Readable},
    el_model::EMCreateCtx,
};

use super::{StructureCreate, VisitBy};

pub struct Repeat<Wire, Tb>
where
    Tb: TreeBuilderFn,
    Wire: AsReadWire,
{
    src: ReadWire<Vec<Wire>>,
    dirty_flag: DirtyFlag,
    tree_builder: Tb,
    map: RefCell<HashMap<*const (), MapItem<Wire::TargetData, Tb::Tree>>>,
    ctx: EMCreateCtx,
}

struct MapItem<Data: ?Sized, Tree> {
    _data: Weak<dyn Readable<Data = Data>>,
    tree: Tree,
    alive: bool,
}

impl<Cp, Wire, Tb> VisitBy<Cp> for Repeat<Wire, Tb>
where
    Wire: AsReadWire + 'static,
    Tb: TreeBuilderFn,
    Tb::Tree: VisitBy<Cp>,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor<Cp>,
    {
        self.update_if_need();
        let map = self.map.borrow();
        for key in &*self.src.read() {
            map[&ptr_of_wire(key)].tree.visit(v)?;
        }
        Ok(())
    }

    fn visit_mut<V>(&mut self, v: &mut V) -> crate::Result<()>
    where
        V: super::VisitorMut<Cp>,
    {
        self.update_if_need();
        let mut map = self.map.borrow_mut();
        for key in &*self.src.read() {
            map.get_mut(&ptr_of_wire(key)).unwrap().tree.visit_mut(v)?;
        }
        Ok(())
    }
}

impl<Wire, Tb> Repeat<Wire, Tb>
where
    Wire: AsReadWire,
    Tb: TreeBuilderFn,
{
    fn update_if_need(&self) {
        if !self.dirty_flag.is_dirty() {
            return;
        }

        let mut map = self.map.borrow_mut();

        for wire in &*self.src.read() {
            let wire = wire.as_read_wire();
            match map.entry(Rc::as_ptr(wire) as _) {
                Entry::Occupied(mut occ) => {
                    occ.get_mut().alive = true;
                }
                Entry::Vacant(vac) => {
                    vac.insert(MapItem {
                        _data: Rc::downgrade(wire),
                        tree: self.tree_builder.build(&self.ctx),
                        alive: true,
                    });
                }
            }
        }

        map.retain(|_, item| {
            let alive = item.alive;
            if alive {
                item.alive = false;
            }
            alive
        });

        self.dirty_flag.set_clean();
    }
}

pub fn repeat<W, F>(vec: ReadWire<Vec<W>>, body: F) -> impl StructureCreate
where
    W: AsReadWire + 'static,
    F: TreeBuilderFn,
{
    |ctx: &EMCreateCtx| Repeat {
        dirty_flag: {
            let flag = DirtyFlag::new();
            flag.wake();
            vec.add_listener(flag.listener());
            flag
        },
        src: vec,
        tree_builder: body,
        map: RefCell::new(HashMap::new()),
        ctx: ctx.clone(),
    }
}

pub trait TreeBuilderFn: 'static {
    type Tree;
    fn build(&self, ctx: &EMCreateCtx) -> Self::Tree;
}

impl<F, Tb> TreeBuilderFn for F
where
    F: Fn() -> Tb + 'static,
    Tb: StructureCreate,
{
    type Tree = Tb::Target;
    fn build(&self, ctx: &EMCreateCtx) -> Self::Tree {
        self().create(ctx)
    }
}

fn ptr_of_wire<W>(w: &W) -> *const ()
where
    W: AsReadWire,
{
    Rc::as_ptr(w.as_read_wire()) as _
}
