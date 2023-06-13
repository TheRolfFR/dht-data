pub fn first_and_last_character(input: &str) -> String {
    let mut result = String::new();
    let (first, rest) = input.split_at(1);

    if let Some(last) = rest.chars().rev().next() {
        result.push(first.chars().next().unwrap());
        for _ in 0..rest.len() - 1 {
            result.push('*');
        }
        result.push(last);
    }
    
    result
}