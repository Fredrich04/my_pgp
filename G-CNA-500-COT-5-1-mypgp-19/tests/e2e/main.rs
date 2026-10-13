use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

type TestResult = Result<(), Box<dyn Error>>;

const XOR_KEY_HEX: &str = "576861742069732064656164206d6179206e6576657220646965";
const XOR_MESSAGE: &str = "You know nothing, Jon Snow\n";
const XOR_CIPHERTEXT_HEX: &str = "20070f2700071c6a4449060a490515164e4e12190b190011063c";

const AES_KEY_HEX: &str = "57696e74657220697320636f6d696e67";
const AES_MESSAGE: &str = "All men must die\n";
const AES_CIPHERTEXT_HEX: &str = "744ce22c385958348f0df26eceb62eef";
const RSA_PUBLIC_KEY: &str = "0101-19bb";
const RSA_PRIVATE_KEY: &str = "9d5b-19bb";
const RSA_CIPHERTEXT_HEX: &str = "8f84";
const PGP_AES_KEY: &str = "57696e74657220697320636f6d696e67:010001-c9f91a9ff3bd6d84005b9cc8448296330bd23480f8cf8b36fd4edd0a8cd925de139a0076b962f4d57f50d6f9e64e7c41587784488f923dd60136c763fd602fb3";
const PGP_AES_ENCRYPTED_KEY: &str = "97f2af4c1b712008c1e46935f446756443a8a700f20581d138e4e6916afe5c5f9b9d6eaa0a870374b686f1a024f9bbb88c23c766654579339caf55afd149d41d";

fn run(args: &[&str], stdin: &str) -> Result<(String, i32), Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_my_pgp"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let _ = child
        .stdin
        .take()
        .ok_or("missing stdin")?
        .write_all(stdin.as_bytes());
    let output = child.wait_with_output()?;
    let code = output.status.code().ok_or("process terminated by signal")?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok((stdout, code))
}

#[test]
fn test_binary_without_crypto_system_fails_with_84() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_my_pgp")).output()?;
    assert_eq!(
        output.status.code(),
        Some(84),
        "every error must exit with 84 (subject p. 1)"
    );
    assert!(
        output.stdout.is_empty(),
        "nothing may be printed on stdout on error, got {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !output.stderr.is_empty(),
        "the error must be explained on stderr (subject p. 1)"
    );
    Ok(())
}

#[test]
fn test_xor_block_mode_matches_the_subject_vector() -> TestResult {
    let (stdout, code) = run(&["xor", "-c", "-b", XOR_KEY_HEX], XOR_MESSAGE)?;
    assert_eq!(code, 0, "the subject XOR vector must succeed");
    assert_eq!(
        stdout.trim_end(),
        XOR_CIPHERTEXT_HEX,
        "subject p. 4: xor -c -b must reproduce the exact vector"
    );
    Ok(())
}

#[test]
fn test_xor_block_mode_round_trips_the_subject_vector() -> TestResult {
    let (ciphertext, _) = run(&["xor", "-c", "-b", XOR_KEY_HEX], XOR_MESSAGE)?;
    let (plaintext, code) = run(&["xor", "-d", "-b", XOR_KEY_HEX], ciphertext.trim_end())?;
    assert_eq!(code, 0, "deciphering the subject vector must succeed");
    assert_eq!(
        plaintext, XOR_MESSAGE,
        "xor -d -b must restore the original message exactly"
    );
    Ok(())
}

#[test]
fn test_xor_block_mode_rejects_a_key_of_the_wrong_size() -> TestResult {
    let (stdout, code) = run(&["xor", "-c", "-b", "5761"], XOR_MESSAGE)?;
    assert_eq!(code, 84, "a key not matching the message size must exit 84");
    assert!(
        stdout.is_empty(),
        "nothing may be printed on stdout on error, got {:?}",
        stdout
    );
    Ok(())
}

#[test]
fn test_aes_block_mode_matches_the_subject_vector() -> TestResult {
    let (stdout, code) = run(&["aes", "-c", "-b", AES_KEY_HEX], AES_MESSAGE)?;
    assert_eq!(code, 0, "the subject AES vector must succeed");
    assert_eq!(
        stdout.trim_end(),
        AES_CIPHERTEXT_HEX,
        "subject p. 5: aes -c -b must reproduce the exact vector"
    );
    Ok(())
}

#[test]
fn test_aes_block_mode_round_trips_the_subject_vector() -> TestResult {
    let (ciphertext, _) = run(&["aes", "-c", "-b", AES_KEY_HEX], AES_MESSAGE)?;
    let (plaintext, code) = run(&["aes", "-d", "-b", AES_KEY_HEX], ciphertext.trim_end())?;
    assert_eq!(code, 0, "deciphering the subject AES vector must succeed");
    assert_eq!(
        plaintext, AES_MESSAGE,
        "aes -d -b must restore the original message exactly"
    );
    Ok(())
}

#[test]
fn test_aes_stream_mode_pads_and_round_trips_a_longer_message() -> TestResult {
    let message = "All men must die, but we are not men.\n";
    let (ciphertext, cipher_code) = run(&["aes", "-c", AES_KEY_HEX], message)?;
    assert_eq!(
        cipher_code, 0,
        "stream mode must accept a multi-block message"
    );
    let (plaintext, decipher_code) = run(&["aes", "-d", AES_KEY_HEX], ciphertext.trim_end())?;
    assert_eq!(decipher_code, 0, "stream mode deciphering must succeed");
    assert!(
        plaintext.starts_with(message.trim_end()),
        "stream mode must restore the original message, padding zeros aside; got {:?}",
        plaintext
    );
    Ok(())
}

#[test]
fn test_aes_rejects_a_key_that_is_not_sixteen_twenty_four_or_thirty_two_bytes() -> TestResult {
    let (stdout, code) = run(&["aes", "-c", "-b", "2a2a2a"], AES_MESSAGE)?;
    assert_eq!(code, 84, "an invalid AES key length must exit 84");
    assert!(
        stdout.is_empty(),
        "nothing may be printed on stdout on error, got {:?}",
        stdout
    );
    Ok(())
}

#[test]
fn test_rsa_generation_matches_the_subject_vector() -> TestResult {
    let (stdout, code) = run(&["rsa", "-g", "d3", "e3"], "")?;
    assert_eq!(code, 0, "the RSA generation vector must succeed");
    assert_eq!(
        stdout, "public key: 0101-19bb\nprivate key: 9d5b-19bb\n",
        "subject RSA generation must preserve the exact key format"
    );
    Ok(())
}

#[test]
fn test_rsa_cipher_and_decipher_match_the_subject_vector() -> TestResult {
    let (ciphertext, cipher_code) = run(&["rsa", "-c", RSA_PUBLIC_KEY], "WF\n")?;
    assert_eq!(cipher_code, 0, "RSA encryption must succeed");
    assert_eq!(
        ciphertext.trim_end(),
        RSA_CIPHERTEXT_HEX,
        "subject RSA encryption vector must match"
    );

    let (plaintext, decipher_code) = run(&["rsa", "-d", RSA_PRIVATE_KEY], RSA_CIPHERTEXT_HEX)?;
    assert_eq!(decipher_code, 0, "RSA decryption must succeed");
    assert_eq!(
        plaintext, "WF\n",
        "RSA decryption must restore the subject message"
    );
    Ok(())
}

#[test]
fn test_pgp_aes_matches_the_subject_two_line_vector() -> TestResult {
    let (output, code) = run(&["pgp-aes", "-c", "-b", PGP_AES_KEY], AES_MESSAGE)?;
    assert_eq!(code, 0, "PGP-AES encryption must succeed");
    assert_eq!(
        output,
        format!("{}\n{}\n", PGP_AES_ENCRYPTED_KEY, AES_CIPHERTEXT_HEX),
        "subject PGP-AES output must contain the encrypted key then message"
    );
    Ok(())
}

#[test]
fn test_pgp_xor_round_trips_with_the_small_rsa_subject_key() -> TestResult {
    let (output, code) = run(&["pgp-xor", "-c", "-b", "01:0101-19bb"], "A")?;
    assert_eq!(code, 0, "PGP-XOR encryption must succeed");
    let mut lines = output.lines();
    let encrypted_key = lines.next().ok_or("missing encrypted PGP key")?;
    let encrypted_message = lines.next().ok_or("missing encrypted PGP message")?;
    assert!(
        lines.next().is_none(),
        "PGP-XOR must emit exactly two lines"
    );

    let private_key = format!("{}:{}", encrypted_key, RSA_PRIVATE_KEY);
    let (piped_plaintext, piped_code) = run(
        &["pgp-xor", "-d", "-b", private_key.as_str()],
        output.as_str(),
    )?;
    assert_eq!(
        piped_code, 0,
        "PGP-XOR must accept its complete two-line output"
    );
    assert_eq!(
        piped_plaintext, "A\n",
        "PGP-XOR complete-output pipeline must restore the message"
    );

    let (plaintext, decipher_code) = run(
        &["pgp-xor", "-d", "-b", private_key.as_str()],
        encrypted_message,
    )?;
    assert_eq!(decipher_code, 0, "PGP-XOR decryption must succeed");
    assert_eq!(
        plaintext, "A\n",
        "PGP-XOR must restore the original message"
    );
    Ok(())
}
