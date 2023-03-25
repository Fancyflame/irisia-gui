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
        fn render(
            $slf: &mut Self,
            _: Self::Props<'_>,
            _: &impl style::StyleContainer,
            __cache_box: &mut $crate::CacheBox,
            __chan_setter: &$crate::event::EventChanSetter,
            _: $crate::structure::Slot<impl Node>,
            mut __content: $crate::element::RenderContent,
            __map: &mut dyn FnMut(::std::option::Option<$crate::primary::Region>) -> $crate::Result<$crate::primary::Region>,
        ) -> $crate::Result<()> {
            $crate::structure::Node::finish(
                $crate::build! {
                    @init(__chan_setter);
                    $($tt)*
                },
                __cache_box,
                __content.inherit(),
                __map(::std::option::Option::None)?
            )
        }
    };
}
