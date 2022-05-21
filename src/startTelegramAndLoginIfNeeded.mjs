import { config } from "./config.mjs";
import { TelegramClient } from "telegram";
import input from "input";
import { StoreSession } from "telegram/sessions/index.js";

const storeSession = new StoreSession("./sessionStorage");

export async function startTelegramAndloginIfNeeded() {
  console.log("Starting...");

  const client = new TelegramClient(
    storeSession,
    config.tg.apiId,
    config.tg.apiHash,
    {
      connectionRetries: 5,
    }
  );

  await client.start({
    phoneNumber: async () => await input.text("Please enter your number: "),
    password: async () => await input.text("Please enter your password: "),
    phoneCode: async () =>
      await input.text("Please enter the code you received: "),
    onError: (err) => console.log(err),
  });

  console.log("You should now be connected.");

  return client;
}
