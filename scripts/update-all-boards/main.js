const Firestore = require("@google-cloud/firestore");
const moment = require("moment");

const firestore = new Firestore();

async function main() {
  const expire_at = moment().add(6, "months").toDate();

  let lastBoard = null;
  let boards;
  do {
    let query = firestore.collection("boards").limit(100);
    if (lastBoard) {
      query = query.startAfter(lastBoard);
    }
    boards = await query.get();
    lastBoard = boards.docs[boards.docs.length - 1];
    boards.forEach((board) => {
      board.ref.update({ expire_at });
      console.log("updated board", { boardId: board.id, expireAt: expire_at });
    });
  } while (boards.docs.length > 0);
}

main();
