use contacts::process_vcard;

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
    assert!(response.contains("General commands"));
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
    assert!(response.contains("You don't have any")); // Changed assertion

    // Add a contact through vcard
    let vcard_data = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Smith\nTEL:+19876543210\nEND:VCARD\n";
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
    assert!(response.contains("You don't have any")); // Changed assertion

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
    assert!(response.contains("You don't have any")); // Changed assertion

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
        "SELECT id as \"id!\", contact_name, contact_user_number FROM contacts WHERE submitter_number = ?",
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

#[sqlx::test]
async fn test_multi_number_contact_selection(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register user first
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Add a contact with multiple numbers through vcard
    let vcard_data = "BEGIN:VCARD\n\
        VERSION:3.0\n\
        FN:Alice Smith\n\
        TEL;TYPE=CELL:+19876543210\n\
        TEL;TYPE=WORK:+19876543211\n\
        TEL;TYPE=HOME:+19876543212\n\
        END:VCARD\n";

    let mut reader = ical::VcardParser::new(vcard_data.as_bytes());
    let vcard = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard).await?;
    assert!(matches!(result, ImportResult::Deferred));

    // Add another contact with multiple numbers
    let vcard_data_2 = "BEGIN:VCARD\n\
        VERSION:3.0\n\
        FN:Bob Jones\n\
        TEL;TYPE=CELL:+19876543220\n\
        TEL;TYPE=WORK:+19876543221\n\
        END:VCARD\n";

    let mut reader = ical::VcardParser::new(vcard_data_2.as_bytes());
    let vcard = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard).await?;
    assert!(matches!(result, ImportResult::Deferred));

    // Check response shows pending contacts with multiple numbers
    let response = send_message(&pool, "+1234567890", "h").await?;
    assert!(response.contains("Alice Smith"));
    assert!(response.contains("Bob Jones"));
    assert!(response.contains("+19876543210")); // Alice's cell
    assert!(response.contains("+19876543220")); // Bob's cell
    assert!(response.contains("CELL"));
    assert!(response.contains("WORK"));

    // Select numbers for both contacts (Alice's WORK and Bob's CELL)
    let response = send_message(&pool, "+1234567890", "confirm 1b, 2a").await?;
    assert!(response.contains("Successfully added 2 contacts"));
    assert!(response.contains("Alice Smith (+19876543211)")); // Work number
    assert!(response.contains("Bob Jones (+19876543220)")); // Cell number

    // Verify contacts list shows the selected numbers
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(response.contains("Alice Smith"));
    assert!(response.contains("Bob Jones"));
    assert!(response.contains("987")); // Area code checks

    // Verify deferred contacts were cleaned up
    let deferred_count = query!(
        "SELECT COUNT(*) as count FROM deferred_contacts WHERE submitter_number = ?",
        "+1234567890"
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(deferred_count.count, 0);

    // Verify pending action was cleaned up
    let pending_count = query!(
        "SELECT COUNT(*) as count FROM pending_actions WHERE submitter_number = ?",
        "+1234567890"
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(pending_count.count, 0);

    // Try invalid selections
    let vcard_data_3 = "BEGIN:VCARD\n\
        VERSION:3.0\n\
        FN:Charlie Brown\n\
        TEL;TYPE=CELL:+19876543230\n\
        TEL;TYPE=WORK:+19876543231\n\
        END:VCARD\n";

    let mut reader = ical::VcardParser::new(vcard_data_3.as_bytes());
    let vcard = reader.next().unwrap();
    let result = process_vcard(&pool, "+1234567890", vcard).await?;
    assert!(matches!(result, ImportResult::Deferred));

    // Test various invalid selections
    let response = send_message(&pool, "+1234567890", "confirm 1c").await?; // Invalid letter
    assert!(response.contains("Invalid letter selection: c"));

    let response = send_message(&pool, "+1234567890", "confirm 2a").await?; // Invalid contact number
    assert!(response.contains("Contact number 2 not found"));

    let response = send_message(&pool, "+1234567890", "confirm abc").await?; // Invalid format
    assert!(response.contains("Invalid selection format: abc"));

    Ok(())
}

#[sqlx::test]
async fn test_group_deletion(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register user
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // First add some contacts to make groups with
    let vcard1 = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Smith\nTEL:+19876543210\nEND:VCARD\n";
    let vcard2 = "BEGIN:VCARD\nVERSION:3.0\nFN:Bob Wilson\nTEL:+19876543211\nEND:VCARD\n";
    let vcard3 = "BEGIN:VCARD\nVERSION:3.0\nFN:Carol Brown\nTEL:+19876543212\nEND:VCARD\n";

    // Add contacts
    let mut reader = ical::VcardParser::new(vcard1.as_bytes());
    process_vcard(&pool, "+1234567890", reader.next().unwrap()).await?;
    let mut reader = ical::VcardParser::new(vcard2.as_bytes());
    process_vcard(&pool, "+1234567890", reader.next().unwrap()).await?;
    let mut reader = ical::VcardParser::new(vcard3.as_bytes());
    process_vcard(&pool, "+1234567890", reader.next().unwrap()).await?;

    // Create first group
    let response = send_message(&pool, "+1234567890", "group Alice, Bob").await?;
    assert!(response.contains("Found these contacts"));
    let response = send_message(&pool, "+1234567890", "confirm 1,2").await?;
    assert!(response.contains("Created group \"group0\""));

    // Create second group
    let response = send_message(&pool, "+1234567890", "group Bob, Carol").await?;
    assert!(response.contains("Found these contacts"));
    let response = send_message(&pool, "+1234567890", "confirm 1,2").await?;
    assert!(response.contains("Created group \"group1\""));

    // Verify groups were created with correct members
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(response.contains("group0 (2 members)"));
    assert!(response.contains("group1 (2 members)"));

    // Test deleting just the first group
    let response = send_message(&pool, "+1234567890", "delete group0").await?;
    assert!(response.contains("Found these groups:"));
    assert!(response.contains("group0 (2 members)"));
    let response = send_message(&pool, "+1234567890", "confirm 1").await?;
    assert!(response.contains("Deleted 1 group"));
    assert!(response.contains("group0 (2 members)"));

    // Verify first group was deleted but second remains
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(!response.contains("group0"));
    assert!(response.contains("group1 (2 members)"));

    // Test deleting by partial name match
    let response = send_message(&pool, "+1234567890", "delete group").await?;
    assert!(response.contains("Found these groups:"));
    assert!(response.contains("group1 (2 members)"));
    let response = send_message(&pool, "+1234567890", "confirm 1").await?;
    assert!(response.contains("Deleted 1 group"));

    // Verify all groups are now deleted
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(!response.contains("group0"));
    assert!(!response.contains("group1"));

    // Verify group members were properly cleaned up
    let count = query!("SELECT COUNT(*) as count FROM group_members",)
        .fetch_one(&pool)
        .await?;
    assert_eq!(count.count, 0);

    // Test deleting non-existent group
    let response = send_message(&pool, "+1234567890", "delete nonexistent").await?;
    assert!(response.contains("No groups or contacts found"));

    // Test separate deletion of group and contact
    // First recreate a group
    let _response = send_message(&pool, "+1234567890", "group Alice, Bob").await?;
    let response = send_message(&pool, "+1234567890", "confirm 1,2").await?;
    assert!(response.contains("Created group \"group0\""));
    assert!(response.contains("Alice"));
    assert!(response.contains("Bob"));

    // Delete contact
    let response = send_message(&pool, "+1234567890", "delete Alice").await?;
    assert!(response.contains("Found these contacts:"));
    assert!(response.contains("Alice Smith"));
    let response = send_message(&pool, "+1234567890", "confirm 1").await?;
    assert!(response.contains("Deleted 1 contact"));

    // Delete group
    let response = send_message(&pool, "+1234567890", "delete group0").await?;
    assert!(response.contains("Found these groups:"));
    assert!(response.contains("group0"));
    let response = send_message(&pool, "+1234567890", "confirm 1").await?;
    assert!(response.contains("Deleted 1 group"));

    // Verify both were deleted
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    assert!(!response.contains("group0"));
    assert!(!response.contains("Alice Smith"));
    assert!(response.contains("Bob Wilson")); // Other contacts remain

    Ok(())
}
#[sqlx::test]
async fn test_group_creation_with_nested_groups(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register main user
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Add some contacts first
    let contacts = [
        ("Alice Smith", "+19876543210"),
        ("Bob Wilson", "+19876543211"),
        ("Carol Brown", "+19876543212"),
        ("David Jones", "+19876543213"),
        ("Eve Adams", "+19876543214"),
    ];

    for (name, number) in contacts {
        let vcard = format!(
            "BEGIN:VCARD\nVERSION:3.0\nFN:{}\nTEL:{}\nEND:VCARD\n",
            name, number
        );
        let mut reader = ical::VcardParser::new(vcard.as_bytes());
        process_vcard(&pool, "+1234567890", reader.next().unwrap()).await?;
    }

    // Create first group with Alice and Bob
    let response = send_message(&pool, "+1234567890", "group Alice, Bob").await?;
    println!("First group search response: {}", response);
    assert!(response.contains("Found these contacts"));
    let response = send_message(&pool, "+1234567890", "confirm 1,2").await?;
    println!("First group creation response: {}", response);
    assert!(response.contains("Created group \"group0\""));
    assert!(response.contains("Alice Smith"));
    assert!(response.contains("Bob Wilson"));

    // Create second group with Carol and David
    let response = send_message(&pool, "+1234567890", "group Carol, David").await?;
    println!("Second group search response: {}", response);
    assert!(response.contains("Found these contacts"));
    let response = send_message(&pool, "+1234567890", "confirm 1,2").await?;
    println!("Second group creation response: {}", response);
    assert!(response.contains("Created group \"group1\""));
    assert!(response.contains("Carol Brown"));
    assert!(response.contains("David Jones"));

    // Verify both groups exist and have correct members
    let response = send_message(&pool, "+1234567890", "contacts").await?;
    println!("Contacts list before final group: {}", response);

    // Now create a new group combining individual contact (Eve) and existing groups
    let response = send_message(&pool, "+1234567890", "group Eve, group0, group1").await?;
    println!("Combined group search response: {}", response);
    assert!(response.contains("Found these groups:"));
    assert!(response.contains("group0 (2 members)"));
    assert!(response.contains("group1 (2 members)"));
    assert!(response.contains("Found these contacts:"));
    assert!(response.contains("Eve Adams"));

    // Confirm creation of combined group
    let response = send_message(&pool, "+1234567890", "confirm 1,2,3").await?;
    println!("Final group creation response: {}", response);
    assert!(response.contains("Created group \"group2\""));

    // Debug query to check group members directly from database
    let members = query!(
        r#"
        SELECT DISTINCT c.contact_name
        FROM group_members gm
        JOIN contacts c ON c.contact_user_number = gm.member_number
        WHERE gm.group_id = (
            SELECT id FROM groups 
            WHERE creator_number = ? AND name = 'group2'
        )
        ORDER BY c.contact_name
        "#,
        "+1234567890"
    )
    .fetch_all(&pool)
    .await?;

    println!("Direct database query for group2 members:");
    for member in &members {
        println!("Member: {}", member.contact_name);
    }

    // Verify all members are included
    assert!(response.contains("Alice Smith"));
    assert!(response.contains("Bob Wilson"));
    assert!(response.contains("Carol Brown"));
    assert!(response.contains("David Jones"));
    assert!(response.contains("Eve Adams"));
    assert!(response.contains("5 members")); // Verify total member count

    Ok(())
}

#[sqlx::test]
async fn test_group_creation_subset_selection(pool: Pool<Sqlite>) -> Result<()> {
    setup_db(&pool).await?;

    // Register user
    send_message(&pool, "+1234567890", "name John Doe").await?;

    // Add some test contacts
    let contacts = [
        ("Alice Smith", "+19876543210"),
        ("Bob Wilson", "+19876543211"),
        ("Carol Brown", "+19876543212"),
        ("David Jones", "+19876543213"),
    ];

    for (name, number) in contacts {
        let vcard = format!(
            "BEGIN:VCARD\nVERSION:3.0\nFN:{}\nTEL:{}\nEND:VCARD\n",
            name, number
        );
        let mut reader = ical::VcardParser::new(vcard.as_bytes());
        process_vcard(&pool, "+1234567890", reader.next().unwrap()).await?;
    }

    // Search for all contacts containing "i" (should match Alice, David)
    let response = send_message(&pool, "+1234567890", "group i").await?;
    assert!(response.contains("Found these contacts"));
    assert!(response.contains("Alice Smith"));
    assert!(response.contains("David Jones"));

    // Only select Alice
    let response = send_message(&pool, "+1234567890", "confirm 1").await?;
    assert!(response.contains("Created group \"group0\""));
    assert!(response.contains("Alice Smith"));
    assert!(!response.contains("David Jones"));

    // Verify group contents directly in database
    let members = query!(
        "SELECT c.contact_name 
         FROM group_members gm
         JOIN contacts c ON c.contact_user_number = gm.member_number
         WHERE gm.group_id = (
             SELECT id FROM groups 
             WHERE creator_number = ? AND name = 'group0'
         )
         ORDER BY c.contact_name",
        "+1234567890"
    )
    .fetch_all(&pool)
    .await?;

    assert_eq!(members.len(), 1);
    assert_eq!(members[0].contact_name, "Alice Smith");

    Ok(())
}
