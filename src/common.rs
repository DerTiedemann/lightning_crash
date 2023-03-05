pub fn byte2hex(data: &[u8]) -> String {
    let mut res: String = "".to_string();
    for val in data {
        res = res + &format!("{val:02X}")
    }
    res
}
