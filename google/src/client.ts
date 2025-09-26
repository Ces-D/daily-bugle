import { OAuth2Client } from "google-auth-library";
import { google } from "googleapis";
import { readFile } from "fs/promises";
import { createServer } from "http";
import openExecutable from "open";

const SCOPES = [
  // See, edit, share, and permanently delete all the calendars you can access using Google Calendar.
  "https://www.googleapis.com/auth/calendar",
  // View and edit events on all your calendars.
  "https://www.googleapis.com/auth/calendar.events",
  // See and change the properties of Google calendars you have access to, and create secondary calendars.
  "https://www.googleapis.com/auth/calendar.calendars",
];
const CALLBACK_PATH = "/callback/google";
const SERVER_PORT = 3000;

export async function googleCalenderClient(credentialFilePath: string) {
  const file = await readFile(credentialFilePath, { encoding: "utf-8" });
  const credentials = JSON.parse(file);

  const oAuth2Client = new OAuth2Client(
    credentials.web.client_id,
    credentials.web.client_secret,
    credentials.web.redirect_uris[0],
  );

  const authorizeUrl = oAuth2Client.generateAuthUrl({
    access_type: "offline",
    scope: SCOPES,
  });

  const validClient = await createCallbackServer(oAuth2Client, authorizeUrl);
  const calendar = google.calendar({ version: "v3", auth: validClient });
  return calendar;
}

function createCallbackServer(
  client: OAuth2Client,
  authURL: string,
): Promise<OAuth2Client> {
  return new Promise((resolve, reject) => {
    const server = createServer(async (req, res) => {
      try {
        if (req.url && req.url.indexOf(CALLBACK_PATH) > -1) {
          // acquire the code from the querystring, and close the web server.
          const qs = new URL(req.url, "http://localhost:3000").searchParams;
          const code = qs.get("code");
          if (typeof code === "string") {
            console.log(`Code is ${code}`);
            res.end("Authentication successful! Please return to the console.");
            const r = await client.getToken(code);
            client.setCredentials(r.tokens);
            console.info("Tokens acquired.");
            resolve(client);
          }
        }
      } catch (e) {
        reject(e);
      }
    }).listen(SERVER_PORT, () => {
      openExecutable(authURL, { wait: false }).then((cp) => cp.unref());
    });
    server.close((e) => {
      if (e instanceof Error) {
        console.info("Error closing Google Callback Server");
        reject(e);
      } else {
        console.info("Successfully closed Google Callback Server");
      }
    });
  });
}
