
#[cfg(unix)]
pub use self::unix::*;

#[cfg(unix)]
mod unix;

#[cfg(win)]
pub use self::win::*;

#[cfg(unix)]
mod win;

#[test]
fn test_errcode() {
    assert_eq!(SUCCESS, SUCCESS);
}
