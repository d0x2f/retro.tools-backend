table! {
    board (id) {
        id -> Bpchar,
        name -> Varchar,
        max_votes -> Int2,
        voting_open -> Bool,
        cards_open -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    card (id) {
        id -> Bpchar,
        rank_id -> Bpchar,
        name -> Varchar,
        description -> Varchar,
        participant_id -> Bpchar,
        created_at -> Timestamp,
    }
}

table! {
    participant (id) {
        id -> Bpchar,
    }
}

table! {
    participant_board (participant_id, board_id) {
        participant_id -> Bpchar,
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
    vote (card_id, participant_id) {
        participant_id -> Bpchar,
        card_id -> Bpchar,
        count -> Int2,
    }
}

joinable!(card -> participant (participant_id));
joinable!(card -> rank (rank_id));
joinable!(participant_board -> board (board_id));
joinable!(participant_board -> participant (participant_id));
joinable!(rank -> board (board_id));
joinable!(vote -> card (card_id));
joinable!(vote -> participant (participant_id));

allow_tables_to_appear_in_same_query!(
    board,
    card,
    participant,
    participant_board,
    rank,
    vote,
);
