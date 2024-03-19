CREATE TABLE friendships (
    number_a text NOT NULL,
    number_b text NOT NULL,
    FOREIGN KEY(number_a) REFERENCES users(number),
    FOREIGN KEY(number_b) REFERENCES users(number)
);
CREATE TABLE friend_requests (
    sender text NOT NULL,
    recipient text NOT NULL,
    FOREIGN KEY(sender) REFERENCES users(number),
    FOREIGN KEY(recipient) REFERENCES users(number)
);
CREATE TABLE friend_blocks (
    blocker text NOT NULL,
    blocked text NOT NULL,
    FOREIGN KEY(blocker) REFERENCES users(number),
    FOREIGN KEY(blocked) REFERENCES users(number)
);