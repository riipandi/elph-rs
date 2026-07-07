/// Remove unpaired UTF-16 surrogates (invalid in UTF-8).
pub fn sanitize_surrogates(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            let code = *c as u32;
            !(0xD800..=0xDFFF).contains(&code)
        })
        .collect()
}
