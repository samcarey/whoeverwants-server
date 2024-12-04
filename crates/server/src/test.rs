use super::*;
use futures::executor::block_on;

fn fixture(pool: Pool<Sqlite>) -> impl Fn(&str) {
    move |message: &str| {
        println!(">'{message}'");
        let response = block_on(process_message(
            &pool,
            SmsMessage {
                From: "TEST_NUMBER".to_string(),
                Body: message.to_string(),
                ..Default::default()
            },
        ))
        .unwrap();
        println!("{response}\n\n");
    }
}

#[sqlx::test]
async fn all(pool: Pool<Sqlite>) -> Result<()> {
    let fixture = fixture(pool);

    fixture("hi");
    fixture("name Sam C.");
    fixture("h");
    fixture("info name");
    fixture("info stop");
    fixture("info  ");
    fixture("info x");
    fixture("info info");
    fixture("info name x");
    fixture("yo");
    fixture("stop");
    fixture("yo");

    Ok(())
}
