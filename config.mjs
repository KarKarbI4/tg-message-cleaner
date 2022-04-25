import dotenv from "dotenv";

dotenv.config();

export const config = {
  tg: {
    apiId: Number(process.env.TG_API_ID),
    apiHash: process.env.TG_API_HASH
  }
}