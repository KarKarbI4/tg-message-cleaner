import { createDialogsProgressBar } from "./createDialogsProgressBar.mjs";

export async function deleteMessages({ messages, client, revoke }) {
  const progressBar = createDialogsProgressBar("Удаляем в диалогах");
  progressBar.start(Object.keys(messages).length, 0);

  for (const [dialogId, dialogMessages] of Object.entries(messages)) {
    progressBar.increment();
    await client.deleteMessages(dialogId, dialogMessages, { revoke });
  }

  progressBar.stop();
}

export async function findMessagesByKeyword({
  client,
  dialogs,
  keyword,
  fromUser,
}) {
  const dialogsProgressBar = createDialogsProgressBar("Ищем в диалогах");

  dialogsProgressBar.start(dialogs.length, 0);
  const channelMessages = {};

  for (const dialog of dialogs) {
    dialogsProgressBar.increment(1, { dialog: dialog.name });
    const messagesIterators = client.iterMessages(dialog.entity, {
      search: keyword,
      fromUser: fromUser,
    });

    for await (const message of messagesIterators) {
      if (channelMessages[dialog.id]) {
        channelMessages[dialog.id].push(message.id);
      } else {
        channelMessages[dialog.id] = [message.id];
      }
    }
  }

  dialogsProgressBar.stop();

  return channelMessages;
}
