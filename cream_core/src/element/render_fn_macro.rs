#[macro_export]
macro_rules! render {
    {
        @init($chan_setter: expr, $cache_box: expr, $content: expr);
        $($tt:tt)*
    } => {
        $crate::structure::Node::finish(
            $crate::build!{
                @init($chan_setter);
                $($tt)*
            },
            $cache_box,
            $content
        )
    };
}

#[macro_export]
macro_rules! render_fn {
    {
        @init($slf:ident);
        $($tt:tt)*
    } => {
        fn render<__CreamMacroS, __CreamMacroC>(
            $slf: &mut Self,
            _: Self::Props<'_>,
            _: &__CreamMacroS,
            __chan_setter: &$crate::event::EventChanSetter,
            __cache_box: &mut $crate::CacheBox,
            _: $crate::structure::Slot<__CreamMacroC>,
            mut __content: $crate::element::RenderContent,
        ) -> $crate::Result<()>
        where
            __CreamMacroS: $crate::style::StyleContainer,
            __CreamMacroC: $crate::structure::Node,
            Self: $crate::element::Element<Children<__CreamMacroC> = __CreamMacroC>,
        {
            $crate::render! {
                @init(__chan_setter, __cache_box, __content.inherit(__content.region()));
                $($tt)*
            }
        }
    };
}
