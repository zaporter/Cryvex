use std::io;
use std::io::Write;

pub fn string_prompt(prompt: &str) -> String {
    println!("{prompt} ");

    let mut component_name = String::new();

    io::stdin()
        .read_line(&mut component_name)
        .expect("Failed to read line");
    let result = component_name.trim().to_string();
    if result.len() > 0 {
        println!("You entered: {}", result);
    }
    result
}

pub fn confirm_prompt(prompt: &str) -> bool {
    print!("{} (y/N): ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.to_lowercase().starts_with('y')
}
