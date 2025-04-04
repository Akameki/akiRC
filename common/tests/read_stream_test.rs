use std::io::Cursor;

use common::stream_handler::blocking_read_until_cr_or_lf;

#[rustfmt::skip]
#[cfg(test)]
#[test]
fn test_read_strings() {
    let input = "hello\rto\na\r\n *brand new* \n\rworld\n\n\r\ngoodbye".as_bytes();
    let mut reader = Cursor::new(input);
    let mut buffer = String::new();

    assert_eq!(blocking_read_until_cr_or_lf(&mut reader, &mut buffer).unwrap().unwrap(), "hello");
    assert_eq!(blocking_read_until_cr_or_lf(&mut reader, &mut buffer).unwrap().unwrap(), "to");
    assert_eq!(blocking_read_until_cr_or_lf(&mut reader, &mut buffer).unwrap().unwrap(), "a");
    assert_eq!(blocking_read_until_cr_or_lf(&mut reader, &mut buffer).unwrap().unwrap(), " *brand new* ");
    assert_eq!(blocking_read_until_cr_or_lf(&mut reader, &mut buffer).unwrap().unwrap(), "world");
    assert_eq!(blocking_read_until_cr_or_lf(&mut reader, &mut buffer).unwrap(), None);
    assert_eq!(buffer, "goodbye");
}
