const Firestore = require("@google-cloud/firestore");

const firestore = new Firestore();

// Deletes all documents in a collection and returns the number deleted
async function deleteCollection(collection) {
  const query = await collection.get();
  query.docs.forEach((snapshot) => snapshot.ref.delete());
  return query.size;
}

async function main() {
  const visited = new Set();

  const iterateBoards = async (doc) => {
    const board = await firestore.doc(doc.ref.parent.parent.path).get();
    if (!board.exists && !visited.has(board.id)) {
      visited.add(board.id);
      const cardsDeleted = await deleteCollection(
        board.ref.collection("cards")
      );
      const columnsDeleted = await deleteCollection(
        board.ref.collection("columns")
      );
      console.log("deleted board orphans", {
        boardId: board.id,
        cardsDeleted,
        columnsDeleted,
      });
    }
  };
  const cards = await firestore.collectionGroup("cards").get();
  const columns = await firestore.collectionGroup("columns").get();
  cards.forEach(iterateBoards);
  columns.forEach(iterateBoards);
}

main();
