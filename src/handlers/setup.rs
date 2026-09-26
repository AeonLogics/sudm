use std::env;
use std::fs::create_dir_all;
use colored::Colorize;

pub fn init_migrations() {
    let current_location = env::current_dir()
        .map_err(|e| format!("Could not read active terminal path: {}", e))
        .unwrap();

    println!("\n {}", "Generating Migration Setup \n".cyan());

    let structures = vec![
        "migrations/Schema",
        "migrations/Functions",
    ];

    for structure in structures {
        let path = current_location.join(structure);

        if path.exists() {
            println!(
                "{} {} {}",
                "✔".green(),
                structure.cyan(),
                "already exists!".yellow()
            );
        } else {
            create_dir_all(&path)
                .map_err(|e| format!("{} {}", "Could not create migrations directory: ".red(), e))
                .unwrap();

            println!(
                "{} {} {}",
                "✔".green(),
                structure.cyan(),
                "created successfully!".cyan()
            );
        }
    }

}