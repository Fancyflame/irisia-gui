pub type EmptyBitsetInferer = BitsetInferer<0, 32>;

#[doc(hidden)]
pub struct BitsetInferer<const U32S: usize, const BITS: usize> {
    _do_not_initialize: (),
}

pub struct BitsetSizeTooLarge;

pub trait BitsetInc {
    type AsBitset;
    type Result: BitsetInc;
}

macro_rules! impl_inc_bit {
    ()=>{};
    ($plus1:tt $($plus:tt)*) => {
        impl<const U32S: usize> BitsetInc
            for BitsetInferer<U32S, {1 $($plus 1)*}>
        where
            BitsetInferer<U32S, {2 $($plus 1)*}>: BitsetInc,
        {
            type Result = BitsetInferer<U32S, {2 $($plus 1)*}>;
            type AsBitset = [u32; U32S];
        }

        impl_inc_bit![$($plus)*];
    };
}

impl_inc_bit![
    +
    ++++++++++
    ++++++++++
    ++++++++++
];

macro_rules! impl_inc_u32 {
    ()=>{};
    ($plus1:tt $($plus:tt)*) => {
        impl BitsetInc for BitsetInferer<{0 $($plus 1)*}, 32>
        {
            type Result = BitsetInferer<{1 $($plus 1)*}, 1>;
            type AsBitset = [u32; {0 $($plus 1)*}];
        }

        impl_inc_u32![$($plus)*];
    };
}

impl_inc_u32![
    ++++++++++++++++++++++++++++++++
    ++++++++++++++++++++++++++++++++
    ++++++++++++++++++++++++++++++++
    +++++++++++++++++++++++++++++++
];

impl BitsetInc for BitsetInferer<127, 32> {
    type Result = BitsetSizeTooLarge;
    type AsBitset = [u32; 127];
}

impl BitsetInc for BitsetSizeTooLarge {
    type Result = Self;
    type AsBitset = ();
}

#[test]
fn test() {
    type MyBitset = BitsetInferer<5, 31>;
    type Inc2 = <<MyBitset as BitsetInc>::Result as BitsetInc>::Result;
    fn foo(x: Inc2) -> BitsetInferer<6, 1> {
        x
    }
}
