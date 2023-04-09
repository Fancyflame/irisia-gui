#[macro_export]
macro_rules! render_fn {
    {
        @init($slf:ident);
        $($tt:tt)*
    } => {
        fn render<'a>(
            $slf: &mut Self,
            mut __frame: $crate::element::Frame<
                Self,
                impl $crate::style::StyleContainer,
                impl VisitIter<Self::ChildProps<'a>>,
            >,
        ) -> $crate::Result<()> {
            $crate::structure::StructureBuilder::into_rendering(
                $crate::build! {
                    @init(__frame.event_dispatcher, __frame.children);
                    $($tt)*
                },
                __frame.cache_box_for_children,
                __frame.content.inherit(),
            ).finish(__frame.drawing_region)
        }
    };
}
