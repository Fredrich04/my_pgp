use my_pgp_lib::io::stdin::{read_message, MAX_INPUT_BYTES};
use my_pgp_lib::{MyPgpError, Plaintext, Result};
use std::io::Cursor;

const TEST_LIMIT_BYTES: usize = 16;

fn read(input: &[u8]) -> Result<Plaintext> {
    read_message(Cursor::new(input.to_vec()), TEST_LIMIT_BYTES)
}

fn assert_reads(input: &[u8], expected: &[u8], reason: &str) -> Result<()> {
    let message = read(input)?;
    assert_eq!(
        message.as_bytes(),
        expected,
        "input {:?}: {}",
        String::from_utf8_lossy(input),
        reason
    );
    Ok(())
}

#[test]
fn test_limit_is_sixty_four_mebibytes() {
    assert_eq!(
        MAX_INPUT_BYTES, 67_108_864,
        "the team fixed the stdin limit at 64 MiB (decision of 2026-09-24)"
    );
}

#[test]
fn test_strips_final_line_feed() -> Result<()> {
    assert_reads(
        b"abc\n",
        b"abc",
        "the final \\n added by echo is not part of the message",
    )
}

#[test]
fn test_strips_final_carriage_return_line_feed() -> Result<()> {
    assert_reads(
        b"abc\r\n",
        b"abc",
        "a Windows \\r\\n line ending is removed as a whole",
    )
}

#[test]
fn test_strips_only_one_final_newline() -> Result<()> {
    assert_reads(b"abc\n\n", b"abc\n", "only one final newline is removed")?;
    assert_reads(
        b"abc\r\n\r\n",
        b"abc\r\n",
        "only one final \\r\\n is removed",
    )
}

#[test]
fn test_keeps_lone_carriage_return_and_inner_newlines() -> Result<()> {
    assert_reads(b"abc\r", b"abc\r", "a lone \\r is data, not a line ending")?;
    assert_reads(b"a\nb", b"a\nb", "inner newlines are part of the message")
}

#[test]
fn test_keeps_input_without_newline() -> Result<()> {
    assert_reads(b"abc", b"abc", "input without newline is kept as is")
}

#[test]
fn test_empty_and_newline_only_inputs_become_empty() -> Result<()> {
    assert_reads(b"", b"", "empty input gives an empty message")?;
    assert_reads(b"\n", b"", "a lone \\n gives an empty message")?;
    assert_reads(b"\r\n", b"", "a lone \\r\\n gives an empty message")
}

#[test]
fn test_accepts_exactly_the_limit() -> Result<()> {
    let message = read(&[b'x'; TEST_LIMIT_BYTES])?;
    assert_eq!(
        message.len(),
        TEST_LIMIT_BYTES,
        "an input of exactly the limit must be accepted"
    );
    Ok(())
}

#[test]
fn test_rejects_one_byte_over_the_limit() {
    let result = read(&[b'x'; TEST_LIMIT_BYTES + 1]);
    assert!(
        matches!(
            result,
            Err(MyPgpError::InputTooLarge {
                limit_bytes: TEST_LIMIT_BYTES
            })
        ),
        "limit + 1 bytes must be rejected (REGLES.md § 17, BLOCKER-16), got {:?}",
        result
    );
}

#[test]
fn test_limit_applies_before_newline_stripping() {
    let mut input = vec![b'x'; TEST_LIMIT_BYTES];
    input.push(b'\n');
    let result = read(&input);
    assert!(
        result.is_err(),
        "the limit counts the raw input, newline included; got {:?}",
        result
    );
}

#[test]
fn test_keeps_binary_bytes() -> Result<()> {
    assert_reads(
        &[0x00, 0xff, 0x80],
        &[0x00, 0xff, 0x80],
        "stream mode encrypts raw bytes",
    )
}

struct FailingReader;

impl std::io::Read for FailingReader {
    fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "stdin closed",
        ))
    }
}

#[test]
fn test_read_error_propagates_as_io_with_its_source() {
    let result = read_message(FailingReader, TEST_LIMIT_BYTES);
    assert!(
        matches!(&result, Err(MyPgpError::Io(source)) if source.kind() == std::io::ErrorKind::BrokenPipe),
        "a failing reader must surface as MyPgpError::Io with the original kind (REGLES.md § 11.5), got {:?}",
        result
    );
    if let Err(error) = result {
        assert!(
            std::error::Error::source(&error).is_some(),
            "the original io::Error must stay reachable through source() (REGLES.md § 3.2)"
        );
    }
}
