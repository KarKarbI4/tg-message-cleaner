import { startTelegramAndloginIfNeeded } from "./startTelegramAndLoginIfNeeded.mjs";
import { findMessagesByKeyword, deleteMessages } from "./messagesOps.mjs";
import { writeToTmpFileSync } from "./writeToTmpFileSync.mjs";

run();

async function run() {
  // process keyword
  const keyword = process.argv[2];
  console.log(process.argv);
  if (!keyword) {
    throw new Error("Provide keyword: npm run start -- keyword");
  }

  // login
  const client = await startTelegramAndloginIfNeeded();
  await client.connect();

  // get dialogs
  const dialogs = await client.iterDialogs({}).collect();
  console.log(`Найдено ${dialogs.length} диалогов`);

  // remove my messages for everyone
  await findAndClearMessages({
    client,
    keyword,
    dialogs,
    fromUser: "me",
    revoke: true,
  });

  // remove others messages for me
  const userDialogs = dialogs.filter((dialog) => dialog.isUser);
  await findAndClearMessages({
    client,
    keyword,
    dialogs: userDialogs,
    fromUser: undefined,
    revoke: false,
  });
}

async function findAndClearMessages({
  client,
  dialogs,
  keyword,
  fromUser,
  revoke,
}) {
  console.log(
    `Finding messages in dialogs by keyword ${keyword}. fromUser: ${fromUser}, revoke: ${revoke}.`
  );
  const messages = await findMessagesByKeyword({
    client,
    dialogs,
    keyword,
    fromUser,
  });

  const totalMessages = Object.values(messages).reduce(
    (agg, messages) => agg + messages.length,
    0
  );

  console.log(
    `Found ${totalMessages} messages in ${Object.keys(messages).length} dialogs`
  );

  console.log("Writing to ./tmp/messages.json");
  writeToTmpFileSync("./messages.json", JSON.stringify(messages));

  console.log("Deleting...");
  await deleteMessages({ client, messages, revoke });
  console.log(`Successfully deleted messages with keyword ${keyword}`);
}
