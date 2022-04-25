import { config } from "./config.mjs";
import { TelegramClient } from "telegram";
import { StoreSession } from "telegram/sessions/index.js";
import input from "input";

const storeSession = new StoreSession("./sessionStorage");

async function startTelegramAndloginIfNeeded() {
  console.log("Loading interactive example...");

  const client = new TelegramClient(storeSession, config.tg.apiId, config.tg.apiHash, {
    connectionRetries: 5,
  });

  await client.start({
    phoneNumber: async () => await input.text("Please enter your number: "),
    password: async () => await input.text("Please enter your password: "),
    phoneCode: async () =>
      await input.text("Please enter the code you received: "),
    onError: (err) => console.log(err),
  });

  console.log("You should now be connected.");
  console.log(client.session.save()); // Save this string to avoid logging in again

  return client;
}

const client = await startTelegramAndloginIfNeeded();

await client.sendMessage("me", { message: "Hello!" });
