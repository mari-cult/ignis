use limine::BaseRevision;
use limine::request::{
    FramebufferRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker,
};

#[unsafe(link_section = ".requests_start_marker")]
static REQUESTS_START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[unsafe(link_section = ".requests")]
pub static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[unsafe(link_section = ".requests_end_marker")]
static REQUESTS_END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
