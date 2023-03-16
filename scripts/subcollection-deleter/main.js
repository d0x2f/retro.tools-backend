const Firestore = require("@google-cloud/firestore");

const firestore = new Firestore();

// Deletes all documents in a collection and returns the number deleted
async function deleteCollection(reference) {
  return firestore
    .collection(reference)
    .get()
    .then((querySnapshot) => {
      querySnapshot.docs.forEach((snapshot) => snapshot.ref.delete());
      return querySnapshot.size;
    });
}

exports.run = async (_event, context) => {
  // Lets be absolutely sure we're supposed to be deleting things
  if (
    context.eventType !== "providers/cloud.firestore/eventTypes/document.delete"
  ) {
    console.log("Received unexcpected event type.", { context });
    return;
  }

  const boardId = context.params.boardId;
  if (!boardId) {
    console.log("No board id found, exiting.");
    return;
  }
  const [cardsDeleted, columnsDeleted] = await Promise.all([
    deleteCollection(`/boards/${boardId}/cards`),
    deleteCollection(`/boards/${boardId}/columns`),
  ]);
  console.log("Deleted board", { boardId, cardsDeleted, columnsDeleted });
};
