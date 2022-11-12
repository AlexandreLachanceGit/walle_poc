require("dotenv").config();
const axios = require("axios");

let url = `https://discord.com/api/v8/applications/${process.env.APP_ID}/commands`;

const headers = {
  Authorization: `Bot ${process.env.BOT_TOKEN}`,
  "Content-Type": "application/json",
};

let command_data = {
  name: "ping",
  type: 1,
  description: "pong",
};

axios.post(url, JSON.stringify(command_data), {
  headers: headers,
});
