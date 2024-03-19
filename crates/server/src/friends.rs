use log::*;
use openapi::apis::configuration::Configuration;
use sqlx::{query, query_as, Pool, Sqlite};

use crate::{send, RowId};

struct Request {
    sender: String,
}

pub async fn accept(
    pool: &Pool<Sqlite>,
    twilio_config: Option<&Configuration>,
    name: &str,
    from: &str,
    request_index: i64,
) -> anyhow::Result<String> {
    Ok({
        let Some(Request { sender }) = query_as!(
            Request,
            "select sender from friend_requests where (recipient, rowid) = (?, ?)",
            from,
            request_index
        )
        .fetch_optional(pool)
        .await?
        else {
            return Ok("Friend request not found".to_string());
        };

        query!(
            "insert into friendships (number_a, number_b) values (?, ?)",
            sender,
            from,
        )
        .execute(pool)
        .await?;

        query!(
            "delete from friend_requests where \
            (sender, recipient) = (?, ?) \
            or (recipient, sender) = (?, ?)",
            sender,
            from,
            from,
            sender,
        )
        .execute(pool)
        .await?;

        let message = format!("{name} has accepted your friend request!");
        send(twilio_config, sender.clone(), message).await?;
        format!("You a now friends with {sender}")
    })
}

pub async fn reject(pool: &Pool<Sqlite>, from: &str, request_index: i64) -> anyhow::Result<String> {
    Ok({
        let Some(Request { sender }) = query_as!(
            Request,
            "select sender from friend_requests where (recipient, rowid) = (?, ?)",
            from,
            request_index
        )
        .fetch_optional(pool)
        .await?
        else {
            return Ok("Friend request not found".to_string());
        };
        query!(
            "delete from friend_requests where \
            (sender, recipient) = (?, ?)",
            sender,
            from,
        )
        .execute(pool)
        .await?;
        format!("Rejected friend request from {sender}")
    })
}

pub async fn friend(
    pool: &Pool<Sqlite>,
    twilio_config: Option<&Configuration>,
    name: &str,
    from: &str,
    friend_number: &str,
) -> anyhow::Result<String> {
    Ok({
        if query!(
            "select count(*) as count from friendships where \
            (number_a, number_b) = (?, ?) or \
            (number_b, number_a) = (?, ?)",
            friend_number,
            from,
            from,
            friend_number
        )
        .fetch_one(pool)
        .await?
        .count
            != 0
        {
            return Ok(format!("You are already friends."));
        }

        if query!(
            "select count(*) as count from friend_requests where \
            (sender, recipient) = (?, ?)",
            from,
            friend_number,
        )
        .fetch_one(pool)
        .await?
        .count
            != 0
        {
            return Ok(format!("You already sent a friend request to them."));
        }

        if let Some(RowId { rowid }) = query_as!(
            RowId,
            "select rowid from friend_requests where \
            (sender, recipient) = (?, ?)",
            friend_number,
            from,
        )
        .fetch_optional(pool)
        .await?
        {
            let accept_response = accept(pool, twilio_config, &name, &from, rowid).await?;
            return Ok(format!(
                "You already received a friend request from them.\n{accept_response}"
            ));
        }

        let response = match query_as!(
            RowId,
            "insert into friend_requests (sender, recipient) values (?, ?) returning rowid",
            from,
            friend_number
        )
        .fetch_one(pool)
        .await
        {
            Ok(RowId { rowid }) => {
                let message = format!(
                    "Friend request received from {name} ({from}). \
                    Reply 'accept {rowid}', 'reject {rowid}', or 'block {rowid}'"
                );
                send(twilio_config, friend_number.to_string(), message).await?;
                "Friend request sent!".to_string()
            }
            Err(error) => {
                error!("{error}");
                "Failed to send friend request!".to_string()
            }
        };
        response
    })
}
