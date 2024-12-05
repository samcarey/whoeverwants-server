use super::*;

async fn setup_db(pool: &Pool<Sqlite>) -> Result<()> {
    query!("PRAGMA foreign_keys = ON").execute(pool).await?;
    Ok(())
}

async fn send_message(pool: &Pool<Sqlite>, from: &str, body: &str) -> Result<String> {
    process_message(
        pool,
        SmsMessage {
            From: from.to_string(),
            Body: body.to_string(),
            ..Default::default()
        },
    )
    .await
}

#[sqlx::test]
async fn test_new_user_registration(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Test initial greeting for unknown number
    let response = send_message(&pool, "+1234567890", "hi").await?;
    assert!(response.contains("Greetings!"));
    assert!(response.contains("name"));

    // Test invalid name command
    let response = send_message(&pool, "+1234567890", "name").await?;
    assert!(response.contains("Reply \"name X\""));

    // Test name too long
    let response = send_message(
        &pool,
        "+1234567890",
        "name This Name Is Way Too Long For The System",
    )
    .await?;
    assert!(response.contains("Please shorten it"));

    // Test successful registration
    let response = send_message(&pool, "+1234567890", "name John Doe").await?;
    assert!(response.contains("Hello, John Doe!"));

    // Verify user was created in database
    let user = query_as!(User, "SELECT * FROM users WHERE number = ?", "+1234567890")
        .fetch_one(&pool)
        .await?;
    assert_eq!(user.name, "John Doe");
    assert_eq!(user.number, "+1234567890");

    Ok(())
}

#[sqlx::test]
async fn test_help_and_info_commands(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register user first
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Test help command
    let response = send_message(&pool, "+1234567890", "h").await?;
    assert!(response.contains("Available commands"));
    assert!(response.contains("name"));
    assert!(response.contains("info"));
    assert!(response.contains("contacts"));

    // Test info command without parameter
    let response = send_message(&pool, "+1234567890", "info").await?;
    assert!(response.contains("Reply \"info X\""));

    // Test info with valid command
    let response = send_message(&pool, "+1234567890", "info name").await?;
    assert!(response.contains("set your preferred name"));

    // Test info with invalid command
    let response = send_message(&pool, "+1234567890", "info invalidcommand").await?;
    assert!(response.contains("Command \"invalidcommand\" not recognized"));

    Ok(())
}

#[sqlx::test]
async fn test_contact_management(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register user
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Test empty contacts list
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(response.contains("haven't added any contacts yet"));

    // Add a contact through vcard
    let vcard_data = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Smith\nTEL:+1987654321\nEND:VCARD\n";
    let mut reader = ical::VcardParser::new(vcard_data.as_bytes());
    let vcard = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard).await?;
    assert!(matches!(result, ImportResult::Added));

    // Check contacts list
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(response.contains("Alice Smith"));
    assert!(response.contains("987"));

    // Test contact deletion - search
    let response = send_message(&pool, "+1234567890", "delete Alice").await?;
    assert!(response.contains("Found these contacts"));
    assert!(response.contains("Alice Smith"));

    // Test contact deletion - confirm
    let response = send_message(&pool, "+1234567890", "confirm 1").await?;
    assert!(response.contains("Deleted 1 contact"));
    assert!(response.contains("Alice Smith"));

    // Verify contact was deleted
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(response.contains("haven't added any contacts yet"));

    Ok(())
}

#[sqlx::test]
async fn test_user_deletion(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register user
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Add a contact
    let vcard_data = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Smith\nTEL:+1987654321\nEND:VCARD\n";
    let mut reader = ical::VcardParser::new(vcard_data.as_bytes());
    let vcard = reader.next().unwrap();
    process_vcard(&pool, "+1234567890", vcard).await?;

    // Test stop command - should cascade delete contacts due to foreign key constraint
    let response = send_message(&pool, "+1234567890", "stop").await?;
    assert!(response.contains("unsubscribed"));

    // Verify user was deleted
    let user = query_as!(User, "SELECT * FROM users WHERE number = ?", "+1234567890")
        .fetch_optional(&pool)
        .await?;
    assert!(user.is_none());

    // Verify contacts were deleted due to foreign key constraint
    let contacts = query_as!(
        Contact,
        "SELECT id as \"id!\", contact_name, contact_number FROM contacts WHERE submitter_number = ?",
        "+1234567890"
    )
    .fetch_all(&pool)
    .await?;
    assert!(contacts.is_empty());

    // Verify new greeting after deletion
    let response = send_message(&pool, "+1234567890", "hi").await?;
    assert!(response.contains("Greetings!"));

    Ok(())
}

#[sqlx::test]
async fn test_contact_updates(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register user
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Add initial contact
    let vcard1_data = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Smith\nTEL:+1987654321\nEND:VCARD\n";
    let mut reader = ical::VcardParser::new(vcard1_data.as_bytes());
    let vcard1 = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard1).await?;
    assert!(matches!(result, ImportResult::Added));

    // Update same contact with new name
    let vcard2_data = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Johnson\nTEL:+1987654321\nEND:VCARD\n";
    let mut reader = ical::VcardParser::new(vcard2_data.as_bytes());
    let vcard2 = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard2).await?;
    assert!(matches!(result, ImportResult::Updated));

    // Verify contact was updated
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(response.contains("Alice Johnson"));
    assert!(!response.contains("Alice Smith"));

    Ok(())
}

#[sqlx::test]
async fn test_malformed_vcard(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register user
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Test malformed vcard
    let malformed_vcard = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Smith\nEND:VCARD\n"; // Missing TEL
    let mut reader = ical::VcardParser::new(malformed_vcard.as_bytes());
    let vcard = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard).await;
    assert!(result.is_err());

    // Verify no contact was added
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(response.contains("haven't added any contacts yet"));

    Ok(())
}

#[sqlx::test]
async fn test_add_contacts_before_registration(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Try to add a contact before registering
    let vcard_data = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Smith\nTEL:+1987654321\nEND:VCARD\n";
    let mut reader = ical::VcardParser::new(vcard_data.as_bytes());
    let vcard = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard).await;

    // Should get an error about registering first
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("set your name first"));

    // Register the user
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Now try adding the contact again
    let mut reader = ical::VcardParser::new(vcard_data.as_bytes());
    let vcard = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard).await?;
    assert!(matches!(result, ImportResult::Added));

    Ok(())
}
