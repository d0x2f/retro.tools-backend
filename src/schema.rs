table! {
    board (id) {
        id -> Bpchar,
        name -> Varchar,
        max_votes -> Int2,
        voting_open -> Bool,
        cards_open -> Bool,
    }
}

table! {
    card (id) {
        id -> Bpchar,
        rank_id -> Bpchar,
        name -> Varchar,
        description -> Varchar,
    }
}

table! {
    participant (id, board_id) {
        id -> Bpchar,
        board_id -> Bpchar,
        owner -> Bool,
    }
}

table! {
    rank (id) {
        id -> Bpchar,
        board_id -> Bpchar,
        name -> Varchar,
    }
}

table! {
    vote (id) {
        id -> Int4,
        card_id -> Bpchar,
        user_token -> Varchar,
    }
}

joinable!(card -> rank (rank_id));
joinable!(participant -> board (board_id));
joinable!(rank -> board (board_id));
joinable!(vote -> card (card_id));

allow_tables_to_appear_in_same_query!(
    board,
    card,
    participant,
    rank,
    vote,
);
