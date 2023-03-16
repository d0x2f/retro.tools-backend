const Firestore = require("@google-cloud/firestore");
const { Logging } = require("@google-cloud/logging");

const logging = new Logging();
const firestore = new Firestore();
const log = logging.log("subcollection-deleter");

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
    log.entry({ context }, "Received unexcpected event type.");
    return;
  }

  const boardId = context.params.boardId;
  if (!boardId) {
    log.entry("No board id found, exiting.");
    return;
  }
  const [cardsDeleted, columnsDeleted] = await Promise.all([
    deleteCollection(`/boards/${boardId}/cards`),
    deleteCollection(`/boards/${boardId}/columns`),
  ]);
  log.entry({ boardId, cardsDeleted, columnsDeleted }, "Deleted board");
};
