use crate::{application::content::GlobalContent, hook::consumer::Consumer};

pub struct ElementDeps<'a, El> {
    inner: &'a Consumer<El>,
    gc: &'a GlobalContent,
}
