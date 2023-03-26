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
            __region: $crate::primary::Region,
            __cache_box_for_children: &mut $crate::CacheBox,
            __chan_setter: &$crate::event::EventChanSetter,
            _: $crate::structure::Slot<impl $crate::structure::StructureBuilder>,
            mut __content: $crate::element::RenderContent,
        ) -> $crate::Result<()> {
            $crate::structure::StructureBuilder::into_rendering(
                $crate::build! {
                    @init(__chan_setter);
                    $($tt)*
                },
                __cache_box_for_children,
                __content.inherit(),
            ).finish(__region)
        }
    };
}
