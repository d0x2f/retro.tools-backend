const firebase = require("firebase-admin");
const { Client } = require("pg");

firebase.initializeApp();
const firestore = firebase.firestore();
const pg = new Client();

async function importBoard(batch, board) {
  console.log(`importing board: ${board.id}`);
  const doc = firestore.collection("boards").doc(board.id);
  return batch.set(doc, {
    name: board.name,
    owner: firestore.collection("participants").doc(board.participant_id),
    data: JSON.stringify(board.data),
    cards_open: board.cards_open,
    voting_open: board.voting_open,
  });
}

async function importColumns(batch, boardId) {
  const res = await pg.query(
    `
    SELECT
      id,
      name,
      data
    FROM
      rank
    WHERE
      board_id = $1
    `,
    [boardId]
  );
  return Promise.all(
    res.rows.map((column, position) => {
      console.log(`importing column: ${column.id}`);
      return batch.set(
        firestore.collection(`boards/${boardId}/columns`).doc(column.id),
        {
          ...column,
          position,
          data: JSON.stringify(column.data),
        }
      );
    })
  );
}

async function importCard(batch, card) {
  console.log(`importing card: ${card.id}`);
  const res = await pg.query(
    `
    SELECT
      participant_id
    FROM
      vote
    WHERE
      card_id = $1
    `,
    [card.id]
  );
  const votes = res.rows.map((row) =>
    firestore.collection("participants").doc(row.participant_id)
  );
  return batch.set(
    firestore.collection(`boards/${card.board_id}/cards`).doc(card.id),
    {
      text: card.description,
      author: card.author,
      owner: firestore.collection("participants").doc(card.participant_id),
      column: firestore
        .collection(`boards/${card.board_id}/columns`)
        .doc(card.rank_id),
      votes,
    }
  );
}

async function importCards(batch, boardId) {
  const res = await pg.query(
    `
    SELECT
      rank.board_id,
      card.id,
      card.rank_id,
      card.description,
      card.author,
      card.participant_id
    FROM
      card
      JOIN rank ON rank.id = card.rank_id
    WHERE
      rank.board_id = $1
    `,
    [boardId]
  );
  for (const card of res.rows) {
    await importCard(batch, card);
  }
}

async function importBoards() {
  const res = await pg.query(
    `
    SELECT
      *
    FROM
      board
      JOIN participant_board ON participant_board.board_id = board.id
    WHERE
      participant_board.owner = true
    `
  );

  for (const board of res.rows) {
    const batch = firestore.batch();
    await importBoard(batch, board);
    await importColumns(batch, board.id);
    await importCards(batch, board.id);
    await batch.commit();
  }
}

async function importParticipant(participant) {
  console.log(`importing participant: ${participant.id}`);
  const res = await pg.query(
    `
    SELECT
      board_id
    FROM
      participant_board
    WHERE
      participant_id = $1
    `,
    [participant.id]
  );
  const boards = res.rows.map((row) =>
    firestore.collection("boards").doc(row.board_id)
  );
  return firestore.collection("participants").doc(participant.id).set({
    boards,
  });
}

async function importParticipants() {
  const res = await pg.query(
    `
    SELECT
      id
    FROM
      participant
    WHERE
      participant.id IN (
        SELECT
          participant_id
        FROM
          participant_board
      )
    `
  );
  for (const participant of res.rows) {
    await importParticipant(participant);
  }
}

(async () => {
  await pg.connect();
  await Promise.all([importParticipants(), importBoards()]);
  await pg.end();
})();
