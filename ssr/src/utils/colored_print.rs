use colored::Colorize;

#[macro_export] // This is crucial!
macro_rules! cprintln {
    ($color:expr, $fmt:expr, $($arg:tt)*) => {{
        let formatted = format!($fmt, $($arg)*);
        match $color {
            "red" => println!("{}", formatted.red()),
            "blue" => println!("{}", formatted.blue()),
            "green" => println!("{}", formatted.green()),
            "yellow" => println!("{}", formatted.yellow()),
            "purple" => println!("{}", formatted.purple()),
            "magenta" => println!("{}", formatted.magenta()),
            "bright_blue" => println!("{}", formatted.bright_blue()),
            "bold" => println!("{}", formatted.bold()),
            "italic" => println!("{}", formatted.italic()),
            "on_blue" => println!("{}", formatted.on_blue()),
            "on_red" => println!("{}", formatted.on_red()),
            "on_green" => println!("{}", formatted.on_green()),
            "on_yellow" => println!("{}", formatted.on_yellow()),
            "on_bright_white" => println!("{}", formatted.on_bright_white()),
            "truecolor" => {
                //Handle truecolor - requires additional parsing of arguments
                //This is a placeholder, needs proper implementation for RGB values
                println!("{}", formatted.truecolor(0,0,0)); //Replace with actual RGB parsing
            },
            "on_truecolor" => {
                //Handle on_truecolor - requires additional parsing of arguments
                //This is a placeholder, needs proper implementation for RGB values
                println!("{}", formatted.on_truecolor(0,0,0)); //Replace with actual RGB parsing
            },
            // "clear" => println!("{:?}", formatted.clear()),
            // "normal" => println!("{}", formatted.normal()),
            _ => println!("{}", formatted), // Default: no color
        }
    }};
}
