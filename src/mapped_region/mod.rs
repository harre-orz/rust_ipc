use {ReadOnly, ReadWrite};
use libc;

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use self::unix::*;

pub enum Advice {
    Normal,
    Sequential,
    Random,
    WillNeed,
    DontNeed,
}

fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

fn adjust_page_offset(offset: isize) -> isize {
    let page_size = page_size() as isize;
    offset - (offset / page_size) * page_size
}

#[test]
fn test_adjust_page_offset() {
    let ps = page_size() as isize;
    assert_eq!(adjust_page_offset(0), 0);
    assert_eq!(adjust_page_offset(1), 1);
    assert_eq!(adjust_page_offset(ps - 1), ps - 1);
    assert_eq!(adjust_page_offset(ps + 0), 0);
    assert_eq!(adjust_page_offset(ps + 1), 1);
}

#[test]
fn test_privilege() {
    assert_eq!(ReadOnly.is_shared(), true);
}
