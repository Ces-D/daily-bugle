// TODO: create calendar events

import { googleCalenderClient } from "./client.js";
import { createBaseCalendar } from "./calendar.js";

const COMMANDS = {
  test: "test",
  createBaseCalendar: "create_base_calendar",
};

const main = async () => {
  const [command, ...args] = process.argv;
  if (Object.values(COMMANDS).includes(command)) {
    switch (command) {
      case COMMANDS.createBaseCalendar: {
        const [credentialFilePath] = args;
        const calendarClient = await googleCalenderClient(credentialFilePath);
        await createBaseCalendar(calendarClient);
        break;
      }
      case COMMANDS.test: {
        const wait = new Promise((resolve) => {
          setTimeout(() => {
            console.log("Successfully tested node wrapper");
            resolve(undefined);
          }, 2_000);
        });
        await wait;
      }
    }
  }
};

main();
