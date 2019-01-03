use libc;

#[cfg(unix)]
mod posix;

#[cfg(unix)]
pub use self::posix::*;

pub enum Advise {
    Normal,
    Sequential,
    Random,
    WillNeed,
    DontNeed,
}

fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

fn adjust_page_offset(offset: usize) -> usize {
    let page_size = page_size();
    offset - (offset / page_size) * page_size
}

#[test]
fn test_adjust_page_offset() {
    let ps = page_size();
    assert_eq!(adjust_page_offset(0), 0);
    assert_eq!(adjust_page_offset(1), 1);
    assert_eq!(adjust_page_offset(ps - 1), ps - 1);
    assert_eq!(adjust_page_offset(ps + 0), 0);
    assert_eq!(adjust_page_offset(ps + 1), 1);
}
