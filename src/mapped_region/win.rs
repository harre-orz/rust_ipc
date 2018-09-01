pub struct MappedRegion<Mode> {
    base: *mut libc::c_void,
    size: usize,
    page_offset: isize,
    handle: Handle,
    mode: PhantomData<Mode>,
}
