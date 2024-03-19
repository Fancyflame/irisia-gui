use std::cell::Cell;

use crate::dep_watch::{bitset::U32Array, inferer::BitsetInc, Bitset};

use super::{
    tracert::{TracertBase, TupleWatch},
    GetBitset, StructureUpdateTo, Updating, VisitBy,
};

pub struct Bind<T: VisitBy> {
    data: T,
    dependents: Cell<Bitset<GetBitset<T>>>,
}

pub struct BindUpdater<F1, F2> {
    pub expr: F1,
    pub updater: F2,
}

impl<T> VisitBy for Bind<T>
where
    T: VisitBy,
    GetBitset<T>: U32Array,
{
    type AddUpdatePoints<Base: BitsetInc> = T::AddUpdatePoints<bitset_inc!(Base)>;
    const UPDATE_POINTS: u32 = 1 + T::UPDATE_POINTS;

    fn visit_by<V>(&self, visitor: &mut V) -> crate::Result<()>
    where
        V: super::VisitOn,
    {
        self.data.visit_by(visitor)
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<R, F1, F2, T, Tu> StructureUpdateTo for BindUpdater<F1, F2>
where
    F1: FnOnce() -> R,
    R: TupleWatch,
    F2: FnOnce(R::Output<GetBitset<T>>) -> Tu,
    Tu: StructureUpdateTo<Target = T>,
{
    type Target = Bind<T>;

    fn create(self, mut info: Updating<impl U32Array>) -> Self::Target {
        let dependents = Cell::new(Bitset::new());
        let child_updater = info.scoped(0, || {
            let r = (self.expr)().trace_all(TracertBase::new(info.call_stack, &dependents));
            (self.updater)(r)
        });

        Bind {
            dependents,
            data: child_updater.create(info.inherit(1, false)),
        }
    }

    fn update(self, target: &mut Self::Target, mut info: Updating<impl U32Array>) {
        if info.no_update::<Self>() {
            return;
        }

        info.step_if(0);

        info.points.union(&target.dependents.take());
        let child_updater = info.scoped(0, || {
            (self.expr)(TracertBase::new(&info.call_stack, &target.dependents))
        });
        child_updater.update(&mut target.data, info.inherit(1, false));
    }
}
