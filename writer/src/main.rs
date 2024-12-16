use consistent_hasher::{LDB, Identifier};

fn u128_to_string(mut number: u128) -> String {
    let mut output = String::new();

    while number != 0 {
        let part = number % 1_000;
        number = number / 1_000;
        output.push_str(&part.to_string());
        output.push('.');
    }
    output.pop();

    let mut vectored: Vec<_> = output.split('.').collect();
    let n = vectored.len();

    for i in 0..2 {
        let temp = vectored[i];
        vectored[i] = vectored[n - i - 1];
        vectored[n - i - 1] = temp;
    }

    return vectored.join(".");

}

struct Node {
    pub ip: String,
}

impl Identifier for Node {
    fn identify(&self) -> usize {
        let ip_parts: Vec<String> = self.ip
            .split('.')
            .map(|part| format!("{:03}", part.parse::<u32>().unwrap_or(0)))
            .collect();

        let x = ip_parts.join("");
        let v = x.parse::<usize>().unwrap();
        v
    }
}

fn main() {
    println!("Starting Rocket server...");

    let n1 = Node {
        ip: "178.12.2.0".to_string()
    };

    let v = n1.identify();
    println!("The thing is {}", v);

    let t = u128_to_string(n1.identify() as u128);
    println!("{}", t);

}