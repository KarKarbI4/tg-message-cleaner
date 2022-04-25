import dotenv from "dotenv";

dotenv.config();

export const config = {
  tg: {
    appId: process.env.TG_APP_ID,
    appHash: process.env.TG_APP_HASH
  }
}