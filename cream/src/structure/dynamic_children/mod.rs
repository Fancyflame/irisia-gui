pub use self::keyed_storage::KeyedStorage;

pub mod keyed_storage;

/*
KeyedStorage:
KeyedStorage是有键多子储存器。每个键对应的值应该是一个与主Application
不同的Application，其中包含了主App的所有可访问域（解引用），另外
还包含了针对Key不同而产生的数据源（Data, Computed, ConstantValue），以及
根据这些新数据生成的元素结构。
e.g.

Application Main{
    field1: Data<u32>,
    field2: Computed<String>,
}

Application AnonymousApp{
    key: ConstantValue<TypeOfKey>,
    computed: Computed<SomeType>,
    _deref: MapWeak<Main>
}
*/
