use bg_ractor::PeriodicActor;
use ractor::concurrency::Duration;
use ractor::Actor;

#[tokio::main]
async fn main() {
    // Create a periodic actor that prints every 5 seconds
    let interval = Duration::from_secs(5); // Change to minutes: Duration::from_secs(60 * n)
    let message = "Hello from periodic actor!".to_string();

    let actor = PeriodicActor::new(interval, message);

    let (actor_ref, handle) = Actor::spawn(Some("periodic_actor".to_string()), actor, ())
        .await
        .expect("Failed to spawn periodic actor");

    println!("Periodic actor started. Let it run for 30 seconds...");

    // Let it run for 30 seconds then stop
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    println!("\nShutting down...");
    actor_ref.stop(None);
    handle.await.expect("Actor failed to exit cleanly");
}
