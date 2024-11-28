use chrono::Local;
use rand::Rng;

pub fn generate_app_reference() -> String {
    let today = Local::now().format("%d%m").to_string();
    let rand_num1: u32 = rand::thread_rng().gen_range(100000..999999);
    let rand_num2: u32 = rand::thread_rng().gen_range(100000..999999);
    format!("HB{}-{}-{}", today, rand_num1, rand_num2)
}
