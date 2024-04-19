use cstr_core::CString;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr() {
        let bytes: &[u8] = &[0x30, 0x31, 0x32]; // Should NOT contain the ending \0
        let s = CString::new(bytes).unwrap();
        assert_eq!(Ok("012"), s.to_str());
        assert_eq!(bytes, s.to_bytes())
    }
}
