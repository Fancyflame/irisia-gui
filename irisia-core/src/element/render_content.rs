use std::{sync::Arc, time::Duration};

use irisia_backend::{window_handle::close_handle::CloseHandle, WinitWindow};

use crate::{
    application::event_comp::{self, focus::SharedFocusing},
    event::EventDispatcher,
    primitive::Region,
    structure::layer,
    CacheBox,
};

pub struct BareContent<'a> {
    pub(crate) window: &'a Arc<WinitWindow>,
    pub(crate) delta_time: Duration,
    pub(crate) window_event_dispatcher: &'a EventDispatcher,
    pub(crate) close_handle: CloseHandle,
    pub(crate) event_comp_builder: event_comp::Builder<'a>,
    pub(crate) focusing: &'a SharedFocusing,
}

impl BareContent<'_> {
    pub fn downgrade_lifetime(&mut self) -> BareContent {
        BareContent {
            window: self.window,
            delta_time: self.delta_time,
            window_event_dispatcher: self.window_event_dispatcher,
            close_handle: self.close_handle,
            event_comp_builder: self.event_comp_builder.downgrade_lifetime(),
            focusing: self.focusing,
        }
    }
}

pub struct RenderContent<'a, 'bdr> {
    pub(crate) bare: BareContent<'a>,
    pub(crate) cache_box_for_children: Option<&'a mut CacheBox>,
    pub(crate) event_comp_index: usize,
    pub(crate) layer_rebuilder: &'a mut layer::Rebuilder<'bdr>,
}

impl<'bdr> RenderContent<'_, 'bdr> {
    pub fn window(&self) -> &Arc<WinitWindow> {
        self.bare.window
    }

    pub fn delta_time(&self) -> Duration {
        self.bare.delta_time
    }

    pub fn set_interact_region(&mut self, region: Region) {
        self.bare
            .event_comp_builder
            .set_interact_region_for(self.event_comp_index, Some(region));
    }

    pub fn clear_interact_region(&mut self) {
        self.bare
            .event_comp_builder
            .set_interact_region_for(self.event_comp_index, None);
    }

    pub(crate) fn downgrade_lifetime(&mut self) -> RenderContent<'_, 'bdr> {
        RenderContent {
            bare: self.bare.downgrade_lifetime(),
            cache_box_for_children: match self.cache_box_for_children {
                Some(ref mut cb) => Some(cb),
                None => None,
            },
            event_comp_index: self.event_comp_index,
            layer_rebuilder: self.layer_rebuilder,
        }
    }
}
